package main

import (
	"context"
	"fmt"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"
	"github.com/aegis-security/aegis-control/internal/api"
	"github.com/aegis-security/aegis-control/internal/auth"
	"github.com/aegis-security/aegis-control/internal/messaging"
	"github.com/aegis-security/aegis-control/internal/scheduler"
	"github.com/aegis-security/aegis-control/internal/store"
	"github.com/gin-gonic/gin"
	"github.com/spf13/cobra"
	"go.uber.org/zap"
)

func main() {
	logger, _ := zap.NewProduction()
	defer logger.Sync()
	rootCmd := &cobra.Command{Use: "aegis-server", Short: "Aegis Control Plane"}
	rootCmd.AddCommand(serverCmd(logger), workerCmd(logger))
	if err := rootCmd.Execute(); err != nil { logger.Fatal("failed", zap.Error(err)) }
}

func serverCmd(logger *zap.Logger) *cobra.Command {
	return &cobra.Command{
		Use: "server", Short: "Start control plane",
		RunE: func(cmd *cobra.Command, args []string) error {
			postgresStore, err := store.NewPostgresStore("postgres://aegis:aegis@localhost:5432/aegis?sslmode=disable", logger)
			if err != nil { return err }
			defer postgresStore.Close()
			msg, err := messaging.NewNATSMessaging("nats://localhost:4222", logger)
			if err != nil { return err }
			defer msg.Close()
			authService := auth.NewService(postgresStore, "change-me-in-production")
			sched := scheduler.NewScheduler(postgresStore, msg, logger)
			ctx, cancel := context.WithCancel(context.Background())
			defer cancel()
			go sched.Start(ctx)
			gin.SetMode(gin.ReleaseMode)
			r := gin.New()
			r.Use(gin.Recovery())
			handler := api.NewHandler(postgresStore, sched, authService, logger)
			handler.RegisterRoutes(r)
			srv := &http.Server{Addr: ":8080", Handler: r}
			quit := make(chan os.Signal, 1)
			signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
			go func() { srv.ListenAndServe() }()
			logger.Info("Server started", zap.String("addr", srv.Addr))
			<-quit
			logger.Info("Shutting down...")
			sched.Stop()
			srv.Shutdown(context.Background())
			return nil
		},
	}
}

func workerCmd(logger *zap.Logger) *cobra.Command {
	return &cobra.Command{
		Use: "worker", Short: "Start fuzzing worker",
		RunE: func(cmd *cobra.Command, args []string) error {
			client, err := worker.NewClient("nats://localhost:4222", logger)
			if err != nil { return err }
			ctx, cancel := context.WithCancel(context.Background())
			defer cancel()
			client.Register(ctx)
			quit := make(chan os.Signal, 1)
			signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
			go func() { <-quit; client.Stop(); cancel() }()
			return client.Start(ctx)
		},
	}
}
