package models

import (
	"time"
	"github.com/google/uuid"
)

type Campaign struct {
	ID uuid.UUID `json:"id" db:"id"`
	Name string `json:"name" db:"name"`
	TargetID uuid.UUID `json:"target_id" db:"target_id"`
	Status CampaignStatus `json:"status" db:"status"`
	TotalExecs uint64 `json:"total_execs" db:"total_execs"`
	ExecsPerSec float64 `json:"execs_per_sec" db:"execs_per_sec"`
	TotalCrashes uint64 `json:"total_crashes" db:"total_crashes"`
	UniqueCrashes uint64 `json:"unique_crashes" db:"unique_crashes"`
	CorpusSize int `json:"corpus_size" db:"corpus_size"`
	CoveragePercent float64 `json:"coverage_percent" db:"coverage_percent"`
	StartTime *time.Time `json:"start_time" db:"start_time"`
	CreatedAt time.Time `json:"created_at" db:"created_at"`
}

type CampaignStatus string
const (
	CampaignStatusPending CampaignStatus = "pending"
	CampaignStatusRunning CampaignStatus = "running"
	CampaignStatusPaused CampaignStatus = "paused"
	CampaignStatusCompleted CampaignStatus = "completed"
	CampaignStatusFailed CampaignStatus = "failed"
)

type Target struct {
	ID uuid.UUID `json:"id" db:"id"`
	Name string `json:"name" db:"name"`
	Type string `json:"type" db:"type"`
	Command string `json:"command" db:"command"`
	Args []string `json:"args" db:"args"`
	InputMode string `json:"input_mode" db:"input_mode"`
	Timeout time.Duration `json:"timeout" db:"timeout"`
	MemoryLimitMB int `json:"memory_limit_mb" db:"memory_limit_mb"`
	Instrumentation InstrumentationConfig `json:"instrumentation" db:"instrumentation"`
	CreatedAt time.Time `json:"created_at" db:"created_at"`
}

type InstrumentationConfig struct {
	UseSancov bool `json:"use_sancov"`
	UseASan bool `json:"use_asan"`
	UseUBSan bool `json:"use_ubsan"`
}

type Worker struct {
	ID uuid.UUID `json:"id" db:"id"`
	Hostname string `json:"hostname" db:"hostname"`
	Address string `json:"address" db:"address"`
	Status WorkerStatus `json:"status" db:"status"`
	LastHeartbeat time.Time `json:"last_heartbeat" db:"last_heartbeat"`
}

type WorkerStatus string
const (
	WorkerStatusOnline WorkerStatus = "online"
	WorkerStatusBusy WorkerStatus = "busy"
	WorkerStatusOffline WorkerStatus = "offline"
)

type Crash struct {
	ID uuid.UUID `json:"id" db:"id"`
	CampaignID uuid.UUID `json:"campaign_id" db:"campaign_id"`
	CrashType string `json:"crash_type" db:"crash_type"`
	StackHash uint64 `json:"stack_hash" db:"stack_hash"`
	CreatedAt time.Time `json:"created_at" db:"created_at"`
}

type User struct {
	ID uuid.UUID `json:"id" db:"id"`
	Email string `json:"email" db:"email"`
	Name string `json:"name" db:"name"`
	Role string `json:"role" db:"role"`
	APIKey string `json:"-" db:"api_key"`
}
