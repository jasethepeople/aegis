package messaging

import (
	"context"
	"encoding/json"
	"fmt"
	"github.com/google/uuid"
	"github.com/nats-io/nats.go"
	"go.uber.org/zap"
)

type NATSMessaging struct {
	nc     *nats.Conn
	js     nats.JetStreamContext
	logger *zap.Logger
}

func NewNATSMessaging(url string, logger *zap.Logger) (*NATSMessaging, error) {
	nc, err := nats.Connect(url, nats.Name("Aegis Control"), nats.RetryOnFailedConnect(true), nats.MaxReconnects(10))
	if err != nil { return nil, fmt.Errorf("nats connect: %w", err) }
	js, err := nc.JetStream()
	if err != nil { nc.Close(); return nil, fmt.Errorf("jetstream: %w", err) }

	streams := []struct{ name, subject string }{
		{"AEGIS_JOBS", "aegis.jobs.*"},
		{"AEGIS_RESULTS", "aegis.results.*"},
		{"AEGIS_COMMANDS", "aegis.commands.*"},
		{"AEGIS_HEARTBEATS", "aegis.heartbeats.*"},
	}
	for _, s := range streams {
		_, err := js.AddStream(&nats.StreamConfig{Name: s.name, Subjects: []string{s.subject}, Retention: nats.WorkQueuePolicy, MaxMsgs: 100000})
		if err != nil && err != nats.ErrStreamNameAlreadyInUse { return nil, err }
	}
	return &NATSMessaging{nc: nc, js: js, logger: logger}, nil
}

func (m *NATSMessaging) PublishJob(ctx context.Context, workerID uuid.UUID, job interface{}) error {
	data, _ := json.Marshal(job)
	_, err := m.js.Publish(fmt.Sprintf("aegis.jobs.%s", workerID.String()), data)
	return err
}

func (m *NATSMessaging) PublishCampaignCommand(ctx context.Context, campaignID uuid.UUID, command string) error {
	data, _ := json.Marshal(map[string]string{"command": command})
	_, err := m.js.Publish(fmt.Sprintf("aegis.commands.%s", campaignID.String()), data)
	return err
}

func (m *NATSMessaging) Close() { m.nc.Close() }
