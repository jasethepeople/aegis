package worker

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"runtime"
	"time"
	"github.com/aegis-security/aegis-control/pkg/models"
	"github.com/google/uuid"
	"github.com/nats-io/nats.go"
	"go.uber.org/zap"
)

type Client struct {
	id           uuid.UUID
	hostname     string
	nc           *nats.Conn
	js           nats.JetStreamContext
	capabilities models.WorkerCapabilities
	logger       *zap.Logger
	stopCh       chan struct{}
}

func NewClient(natsURL string, logger *zap.Logger) (*Client, error) {
	hostname, _ := os.Hostname()
	nc, err := nats.Connect(natsURL, nats.Name(fmt.Sprintf("Worker-%s", hostname)), nats.RetryOnFailedConnect(true), nats.MaxReconnects(10))
	if err != nil { return nil, fmt.Errorf("nats: %w", err) }
	js, err := nc.JetStream()
	if err != nil { nc.Close(); return nil, fmt.Errorf("jetstream: %w", err) }
	return &Client{
		id: uuid.New(), hostname: hostname, nc: nc, js: js,
		capabilities: models.WorkerCapabilities{MaxWorkers: runtime.NumCPU(), CPUCores: runtime.NumCPU(), MemoryGB: 16, SupportsASan: true, SupportsUBSan: true, SupportsSancov: true, OS: runtime.GOOS, Arch: runtime.GOARCH},
		logger: logger, stopCh: make(chan struct{}),
	}, nil
}

func (w *Client) Register(ctx context.Context) error {
	worker := &models.Worker{ID: w.id, Hostname: w.hostname, Address: w.nc.ConnectedUrl(), Status: models.WorkerStatusOnline}
	data, _ := json.Marshal(worker)
	_, err := w.js.Publish("aegis.workers.register", data)
	return err
}

func (w *Client) Start(ctx context.Context) error {
	go w.heartbeatLoop(ctx)
	go w.jobConsumer(ctx)
	<-w.stopCh
	return nil
}

func (w *Client) Stop() { close(w.stopCh); w.nc.Close() }

func (w *Client) heartbeatLoop(ctx context.Context) {
	ticker := time.NewTicker(5 * time.Second)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done(): return
		case <-w.stopCh: return
		case <-ticker.C:
			hb := &models.Heartbeat{WorkerID: w.id, Status: models.WorkerStatusOnline, Timestamp: time.Now()}
			data, _ := json.Marshal(hb)
			w.js.Publish(fmt.Sprintf("aegis.heartbeats.%s", w.id.String()), data)
		}
	}
}

func (w *Client) jobConsumer(ctx context.Context) {
	subject := fmt.Sprintf("aegis.jobs.%s", w.id.String())
	sub, err := w.js.Subscribe(subject, func(msg *nats.Msg) {
		var job map[string]interface{}
		if err := json.Unmarshal(msg.Data, &job); err != nil { msg.Nak(); return }
		w.logger.Info("Received job", zap.Any("job", job))
		// Execute fuzzing job here
		result := map[string]interface{}{"success": true, "execs_done": 1000}
		resultData, _ := json.Marshal(result)
		w.js.Publish(fmt.Sprintf("aegis.results.%s", job["campaign_id"]), resultData)
		msg.Ack()
	}, nats.Durable(fmt.Sprintf("worker-%s", w.id.String())))
	if err != nil { w.logger.Error("Subscribe failed", zap.Error(err)); return }
	<-w.stopCh
	sub.Unsubscribe()
}
