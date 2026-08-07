package scheduler

import (
	"context"
	"fmt"
	"sync"
	"time"
	"github.com/aegis-security/aegis-control/pkg/models"
	"github.com/google/uuid"
	"go.uber.org/zap"
)

type Scheduler struct {
	store     Store
	messaging Messaging
	logger    *zap.Logger
	mu        sync.RWMutex
	campaigns map[uuid.UUID]*CampaignState
	workers   map[uuid.UUID]*WorkerState
	stopCh    chan struct{}
}

type Store interface {
	GetCampaign(ctx context.Context, id uuid.UUID) (*models.Campaign, error)
	UpdateCampaign(ctx context.Context, c *models.Campaign) error
	ListWorkers(ctx context.Context) ([]*models.Worker, error)
	UpdateWorkerHeartbeat(ctx context.Context, id uuid.UUID, status models.WorkerStatus, campaignID *uuid.UUID) error
}

type Messaging interface {
	PublishJob(ctx context.Context, workerID uuid.UUID, job interface{}) error
	PublishCampaignCommand(ctx context.Context, campaignID uuid.UUID, command string) error
}

type CampaignState struct {
	Campaign    *models.Campaign
	Workers     []uuid.UUID
	JobsRunning int
	LastUpdate  time.Time
}

type WorkerState struct {
	Worker        *models.Worker
	CurrentJob    *uuid.UUID
	LastHeartbeat time.Time
	ExecsPerSec   float64
	Healthy       bool
}

func NewScheduler(store Store, messaging Messaging, logger *zap.Logger) *Scheduler {
	return &Scheduler{
		store: store, messaging: messaging, logger: logger,
		campaigns: make(map[uuid.UUID]*CampaignState),
		workers:   make(map[uuid.UUID]*WorkerState),
		stopCh:    make(chan struct{}),
	}
}

func (s *Scheduler) Start(ctx context.Context) error {
	s.logger.Info("Starting scheduler")
	go s.heartbeatMonitor(ctx)
	go s.dispatchLoop(ctx)
	go s.metricsLoop(ctx)
	<-s.stopCh
	return nil
}

func (s *Scheduler) Stop() { close(s.stopCh) }

func (s *Scheduler) RegisterCampaign(ctx context.Context, campaignID uuid.UUID) error {
	campaign, err := s.store.GetCampaign(ctx, campaignID)
	if err != nil { return fmt.Errorf("get campaign: %w", err) }
	s.mu.Lock()
	defer s.mu.Unlock()
	s.campaigns[campaignID] = &CampaignState{Campaign: campaign, Workers: []uuid.UUID{}, JobsRunning: 0, LastUpdate: time.Now()}
	s.logger.Info("Campaign registered", zap.String("campaign_id", campaignID.String()), zap.String("name", campaign.Name))
	return s.assignWorkers(ctx, campaignID)
}

func (s *Scheduler) assignWorkers(ctx context.Context, campaignID uuid.UUID) error {
	state := s.campaigns[campaignID]
	if state == nil { return fmt.Errorf("campaign not found") }
	workers, err := s.store.ListWorkers(ctx)
	if err != nil { return err }
	needed := 4 // default workers
	assigned := 0
	for _, worker := range workers {
		if assigned >= needed { break }
		if worker.CurrentCampaign != nil && *worker.CurrentCampaign != campaignID { continue }
		worker.CurrentCampaign = &campaignID
		if err := s.store.UpdateWorkerHeartbeat(ctx, worker.ID, models.WorkerStatusBusy, &campaignID); err != nil {
			s.logger.Warn("Failed to assign worker", zap.Error(err))
			continue
		}
		state.Workers = append(state.Workers, worker.ID)
		s.workers[worker.ID] = &WorkerState{Worker: worker, Healthy: true, LastHeartbeat: time.Now()}
		assigned++
	}
	s.logger.Info("Workers assigned", zap.String("campaign_id", campaignID.String()), zap.Int("assigned", assigned))
	return nil
}

func (s *Scheduler) dispatchLoop(ctx context.Context) {
	ticker := time.NewTicker(100 * time.Millisecond)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done(): return
		case <-s.stopCh: return
		case <-ticker.C: s.dispatchJobs(ctx)
		}
	}
}

func (s *Scheduler) dispatchJobs(ctx context.Context) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	for campaignID, state := range s.campaigns {
		if state.Campaign.Status != models.CampaignStatusRunning { continue }
		if state.JobsRunning >= len(state.Workers)*2 { continue }
		for _, workerID := range state.Workers {
			if ws, ok := s.workers[workerID]; ok && ws.CurrentJob == nil && ws.Healthy {
				_ = s.messaging.PublishJob(ctx, workerID, map[string]interface{}{"campaign_id": campaignID, "type": "fuzz"})
				ws.CurrentJob = &campaignID
				state.JobsRunning++
			}
		}
	}
}

func (s *Scheduler) heartbeatMonitor(ctx context.Context) {
	ticker := time.NewTicker(5 * time.Second)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done(): return
		case <-s.stopCh: return
		case <-ticker.C: s.checkWorkerHealth(ctx)
		}
	}
}

func (s *Scheduler) checkWorkerHealth(ctx context.Context) {
	s.mu.Lock()
	defer s.mu.Unlock()
	now := time.Now()
	for workerID, state := range s.workers {
		if now.Sub(state.LastHeartbeat) > 30*time.Second && state.Healthy {
			state.Healthy = false
			s.logger.Warn("Worker unhealthy", zap.String("worker_id", workerID.String()))
			s.store.UpdateWorkerHeartbeat(ctx, workerID, models.WorkerStatusOffline, nil)
		}
	}
}

func (s *Scheduler) HandleHeartbeat(heartbeat *models.Heartbeat) {
	s.mu.Lock()
	defer s.mu.Unlock()
	if state, ok := s.workers[heartbeat.WorkerID]; ok {
		state.LastHeartbeat = heartbeat.Timestamp
		state.ExecsPerSec = heartbeat.ExecsPerSec
		state.Healthy = true
	}
}

func (s *Scheduler) metricsLoop(ctx context.Context) {
	ticker := time.NewTicker(10 * time.Second)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done(): return
		case <-s.stopCh: return
		case <-ticker.C:
		}
	}
}

func (s *Scheduler) PauseCampaign(ctx context.Context, campaignID uuid.UUID) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if state, ok := s.campaigns[campaignID]; ok {
		state.Campaign.Status = models.CampaignStatusPaused
		s.store.UpdateCampaign(ctx, state.Campaign)
		for _, workerID := range state.Workers {
			s.messaging.PublishCampaignCommand(ctx, workerID, "pause")
		}
	}
	return nil
}

func (s *Scheduler) StopCampaign(ctx context.Context, campaignID uuid.UUID) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if state, ok := s.campaigns[campaignID]; ok {
		state.Campaign.Status = models.CampaignStatusCompleted
		s.store.UpdateCampaign(ctx, state.Campaign)
		for _, workerID := range state.Workers {
			s.messaging.PublishCampaignCommand(ctx, workerID, "stop")
		}
	}
	return nil
}
