package store

import (
	"context"
	"database/sql"
	"fmt"
	"time"
	"github.com/aegis-security/aegis-control/pkg/models"
	"github.com/google/uuid"
	_ "github.com/lib/pq"
	"go.uber.org/zap"
)

type PostgresStore struct {
	db *sql.DB
	logger *zap.Logger
}

func NewPostgresStore(connString string, logger *zap.Logger) (*PostgresStore, error) {
	db, err := sql.Open("postgres", connString)
	if err != nil { return nil, fmt.Errorf("open db: %w", err) }
	db.SetMaxOpenConns(25)
	db.SetMaxIdleConns(10)
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	if err := db.PingContext(ctx); err != nil { return nil, fmt.Errorf("ping db: %w", err) }
	return &PostgresStore{db: db, logger: logger}, nil
}

func (s *PostgresStore) Close() error { return s.db.Close() }
func (s *PostgresStore) HealthCheck(ctx context.Context) error { return s.db.PingContext(ctx) }

func (s *PostgresStore) CreateCampaign(ctx context.Context, c *models.Campaign) error {
	_, err := s.db.ExecContext(ctx, `INSERT INTO campaigns (id, name, target_id, status, total_execs, execs_per_sec, total_crashes, unique_crashes, corpus_size, coverage_percent, created_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,NOW())`,
		c.ID, c.Name, c.TargetID, c.Status, c.TotalExecs, c.ExecsPerSec, c.TotalCrashes, c.UniqueCrashes, c.CorpusSize, c.CoveragePercent)
	return err
}

func (s *PostgresStore) GetCampaign(ctx context.Context, id uuid.UUID) (*models.Campaign, error) {
	var c models.Campaign
	err := s.db.QueryRowContext(ctx, `SELECT id, name, target_id, status, total_execs, execs_per_sec, total_crashes, unique_crashes, corpus_size, coverage_percent, start_time, created_at FROM campaigns WHERE id=$1`, id).Scan(
		&c.ID, &c.Name, &c.TargetID, &c.Status, &c.TotalExecs, &c.ExecsPerSec, &c.TotalCrashes, &c.UniqueCrashes, &c.CorpusSize, &c.CoveragePercent, &c.StartTime, &c.CreatedAt)
	if err != nil { return nil, err }
	return &c, nil
}

func (s *PostgresStore) UpdateCampaign(ctx context.Context, c *models.Campaign) error {
	_, err := s.db.ExecContext(ctx, `UPDATE campaigns SET name=$1, status=$2, total_execs=$3, execs_per_sec=$4, total_crashes=$5, unique_crashes=$6, corpus_size=$7, coverage_percent=$8 WHERE id=$9`,
		c.Name, c.Status, c.TotalExecs, c.ExecsPerSec, c.TotalCrashes, c.UniqueCrashes, c.CorpusSize, c.CoveragePercent, c.ID)
	return err
}

func (s *PostgresStore) ListCampaigns(ctx context.Context) ([]*models.Campaign, error) {
	rows, err := s.db.QueryContext(ctx, `SELECT id, name, target_id, status, total_execs, execs_per_sec, total_crashes, unique_crashes, corpus_size, coverage_percent, start_time, created_at FROM campaigns ORDER BY created_at DESC`)
	if err != nil { return nil, err }
	defer rows.Close()
	var campaigns []*models.Campaign
	for rows.Next() {
		var c models.Campaign
		rows.Scan(&c.ID, &c.Name, &c.TargetID, &c.Status, &c.TotalExecs, &c.ExecsPerSec, &c.TotalCrashes, &c.UniqueCrashes, &c.CorpusSize, &c.CoveragePercent, &c.StartTime, &c.CreatedAt)
		campaigns = append(campaigns, &c)
	}
	return campaigns, rows.Err()
}

func (s *PostgresStore) CreateTarget(ctx context.Context, t *models.Target) error {
	_, err := s.db.ExecContext(ctx, `INSERT INTO targets (id, name, type, command, args, input_mode, timeout_seconds, memory_limit_mb, created_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,NOW())`,
		t.ID, t.Name, t.Type, t.Command, t.Args, t.InputMode, int(t.Timeout.Seconds()), t.MemoryLimitMB)
	return err
}

func (s *PostgresStore) ListTargets(ctx context.Context) ([]*models.Target, error) {
	rows, err := s.db.QueryContext(ctx, `SELECT id, name, type, command, args, input_mode, timeout_seconds, memory_limit_mb, created_at FROM targets ORDER BY created_at DESC`)
	if err != nil { return nil, err }
	defer rows.Close()
	var targets []*models.Target
	for rows.Next() {
		var t models.Target
		var timeoutSec int
		rows.Scan(&t.ID, &t.Name, &t.Type, &t.Command, &t.Args, &t.InputMode, &timeoutSec, &t.MemoryLimitMB, &t.CreatedAt)
		t.Timeout = time.Duration(timeoutSec) * time.Second
		targets = append(targets, &t)
	}
	return targets, rows.Err()
}

func (s *PostgresStore) RegisterWorker(ctx context.Context, w *models.Worker) error {
	_, err := s.db.ExecContext(ctx, `INSERT INTO workers (id, hostname, address, status, last_heartbeat, registered_at) VALUES ($1,$2,$3,$4,NOW(),NOW()) ON CONFLICT (id) DO UPDATE SET hostname=$2, address=$3, status=$4, last_heartbeat=NOW()`,
		w.ID, w.Hostname, w.Address, w.Status)
	return err
}

func (s *PostgresStore) ListWorkers(ctx context.Context) ([]*models.Worker, error) {
	rows, err := s.db.QueryContext(ctx, `SELECT id, hostname, address, status, last_heartbeat FROM workers ORDER BY last_heartbeat DESC`)
	if err != nil { return nil, err }
	defer rows.Close()
	var workers []*models.Worker
	for rows.Next() {
		var w models.Worker
		rows.Scan(&w.ID, &w.Hostname, &w.Address, &w.Status, &w.LastHeartbeat)
		workers = append(workers, &w)
	}
	return workers, rows.Err()
}

func (s *PostgresStore) CreateCrash(ctx context.Context, c *models.Crash) error {
	_, err := s.db.ExecContext(ctx, `INSERT INTO crashes (id, campaign_id, crash_type, stack_hash, created_at) VALUES ($1,$2,$3,$4,NOW())`,
		c.ID, c.CampaignID, c.CrashType, c.StackHash)
	return err
}

func (s *PostgresStore) ListCrashes(ctx context.Context, campaignID uuid.UUID) ([]*models.Crash, error) {
	rows, err := s.db.QueryContext(ctx, `SELECT id, campaign_id, crash_type, stack_hash, created_at FROM crashes WHERE campaign_id=$1 ORDER BY created_at DESC`, campaignID)
	if err != nil { return nil, err }
	defer rows.Close()
	var crashes []*models.Crash
	for rows.Next() {
		var c models.Crash
		rows.Scan(&c.ID, &c.CampaignID, &c.CrashType, &c.StackHash, &c.CreatedAt)
		crashes = append(crashes, &c)
	}
	return crashes, rows.Err()
}
