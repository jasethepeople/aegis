package api

import (
	"net/http"
	"github.com/aegis-security/aegis-control/internal/auth"
	"github.com/aegis-security/aegis-control/internal/scheduler"
	"github.com/aegis-security/aegis-control/internal/store"
	"github.com/aegis-security/aegis-control/pkg/models"
	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"go.uber.org/zap"
)

type Handler struct {
	store     store.Store
	scheduler *scheduler.Scheduler
	auth      *auth.Service
	logger    *zap.Logger
}

func NewHandler(store store.Store, sched *scheduler.Scheduler, authService *auth.Service, logger *zap.Logger) *Handler {
	return &Handler{store: store, scheduler: sched, auth: authService, logger: logger}
}

func (h *Handler) RegisterRoutes(r *gin.Engine) {
	r.GET("/health", h.HealthCheck)
	v1 := r.Group("/api/v1")
	v1.POST("/auth/login", h.Login)
	auth := v1.Group("/")
	auth.Use(h.authMiddleware())
	{
		auth.GET("/campaigns", h.ListCampaigns)
		auth.POST("/campaigns", h.CreateCampaign)
		auth.GET("/campaigns/:id", h.GetCampaign)
		auth.POST("/campaigns/:id/start", h.StartCampaign)
		auth.POST("/campaigns/:id/pause", h.PauseCampaign)
		auth.POST("/campaigns/:id/stop", h.StopCampaign)
		auth.GET("/targets", h.ListTargets)
		auth.POST("/targets", h.CreateTarget)
		auth.GET("/workers", h.ListWorkers)
		auth.GET("/crashes", h.ListCrashes)
	}
}

func (h *Handler) Login(c *gin.Context) {
	var req struct{ Email string `json:"email"`; Password string `json:"password"` }
	if err := c.ShouldBindJSON(&req); err != nil { c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()}); return }
	token, err := h.auth.Login(c.Request.Context(), req.Email, req.Password)
	if err != nil { c.JSON(http.StatusUnauthorized, gin.H{"error": "invalid credentials"}); return }
	c.JSON(http.StatusOK, gin.H{"token": token})
}

func (h *Handler) ListCampaigns(c *gin.Context) {
	campaigns, err := h.store.ListCampaigns(c.Request.Context())
	if err != nil { c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()}); return }
	c.JSON(http.StatusOK, gin.H{"campaigns": campaigns})
}

func (h *Handler) CreateCampaign(c *gin.Context) {
	var req struct{ Name string `json:"name"`; TargetID string `json:"target_id"` }
	if err := c.ShouldBindJSON(&req); err != nil { c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()}); return }
	targetID, _ := uuid.Parse(req.TargetID)
	campaign := &models.Campaign{ID: uuid.New(), Name: req.Name, TargetID: targetID, Status: models.CampaignStatusPending}
	if err := h.store.CreateCampaign(c.Request.Context(), campaign); err != nil { c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()}); return }
	c.JSON(http.StatusCreated, campaign)
}

func (h *Handler) GetCampaign(c *gin.Context) {
	id, err := uuid.Parse(c.Param("id"))
	if err != nil { c.JSON(http.StatusBadRequest, gin.H{"error": "invalid id"}); return }
	campaign, err := h.store.GetCampaign(c.Request.Context(), id)
	if err != nil { c.JSON(http.StatusNotFound, gin.H{"error": "not found"}); return }
	c.JSON(http.StatusOK, campaign)
}

func (h *Handler) StartCampaign(c *gin.Context) {
	id, _ := uuid.Parse(c.Param("id"))
	campaign, _ := h.store.GetCampaign(c.Request.Context(), id)
	campaign.Status = models.CampaignStatusRunning
	h.store.UpdateCampaign(c.Request.Context(), campaign)
	h.scheduler.RegisterCampaign(c.Request.Context(), id)
	c.JSON(http.StatusOK, gin.H{"status": "started"})
}

func (h *Handler) PauseCampaign(c *gin.Context) {
	id, _ := uuid.Parse(c.Param("id"))
	h.scheduler.PauseCampaign(c.Request.Context(), id)
	c.JSON(http.StatusOK, gin.H{"status": "paused"})
}

func (h *Handler) StopCampaign(c *gin.Context) {
	id, _ := uuid.Parse(c.Param("id"))
	h.scheduler.StopCampaign(c.Request.Context(), id)
	c.JSON(http.StatusOK, gin.H{"status": "stopped"})
}

func (h *Handler) ListTargets(c *gin.Context) {
	targets, err := h.store.ListTargets(c.Request.Context())
	if err != nil { c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()}); return }
	c.JSON(http.StatusOK, gin.H{"targets": targets})
}

func (h *Handler) CreateTarget(c *gin.Context) {
	var target models.Target
	if err := c.ShouldBindJSON(&target); err != nil { c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()}); return }
	target.ID = uuid.New()
	if err := h.store.CreateTarget(c.Request.Context(), &target); err != nil { c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()}); return }
	c.JSON(http.StatusCreated, target)
}

func (h *Handler) ListWorkers(c *gin.Context) {
	workers, err := h.store.ListWorkers(c.Request.Context())
	if err != nil { c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()}); return }
	c.JSON(http.StatusOK, gin.H{"workers": workers})
}

func (h *Handler) ListCrashes(c *gin.Context) {
	campaignID, _ := uuid.Parse(c.Query("campaign_id"))
	crashes, err := h.store.ListCrashes(c.Request.Context(), campaignID)
	if err != nil { c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()}); return }
	c.JSON(http.StatusOK, gin.H{"crashes": crashes})
}

func (h *Handler) HealthCheck(c *gin.Context) {
	if err := h.store.HealthCheck(c.Request.Context()); err != nil {
		c.JSON(http.StatusServiceUnavailable, gin.H{"status": "unhealthy", "error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, gin.H{"status": "healthy"})
}

func (h *Handler) authMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		authHeader := c.GetHeader("Authorization")
		if authHeader == "" { c.JSON(http.StatusUnauthorized, gin.H{"error": "missing auth"}); c.Abort(); return }
		token := authHeader
		if len(authHeader) > 7 && authHeader[:7] == "Bearer " { token = authHeader[7:] }
		user, err := h.auth.ValidateToken(token)
		if err != nil { c.JSON(http.StatusUnauthorized, gin.H{"error": "invalid token"}); c.Abort(); return }
		c.Set("user", user)
		c.Next()
	}
}
