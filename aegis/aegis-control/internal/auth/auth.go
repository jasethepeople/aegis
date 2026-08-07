package auth

import (
	"context"
	"fmt"
	"time"
	"github.com/aegis-security/aegis-control/pkg/models"
	"github.com/golang-jwt/jwt/v5"
	"github.com/google/uuid"
	"golang.org/x/crypto/bcrypt"
)

type Service struct {
	store Store
	secret []byte
}

type Store interface {
	CreateUser(ctx context.Context, user *models.User, passwordHash string) error
	GetUserByEmail(ctx context.Context, email string) (*models.User, error)
	GetUserByAPIKey(ctx context.Context, apiKey string) (*models.User, error)
}

func NewService(store Store, jwtSecret string) *Service {
	return &Service{store: store, secret: []byte(jwtSecret)}
}

func (s *Service) Login(ctx context.Context, email, password string) (string, error) {
	user, err := s.store.GetUserByEmail(ctx, email)
	if err != nil { return "", fmt.Errorf("invalid credentials") }
	if err := bcrypt.CompareHashAndPassword([]byte(user.APIKey), []byte(password)); err != nil {
		return "", fmt.Errorf("invalid credentials")
	}
	return s.generateToken(user)
}

func (s *Service) generateToken(user *models.User) (string, error) {
	claims := jwt.MapClaims{
		"sub": user.ID.String(), "email": user.Email, "name": user.Name,
		"role": user.Role, "iat": time.Now().Unix(), "exp": time.Now().Add(24*time.Hour).Unix(),
	}
	return jwt.NewWithClaims(jwt.SigningMethodHS256, claims).SignedString(s.secret)
}

func (s *Service) ValidateToken(tokenString string) (*models.User, error) {
	token, err := jwt.Parse(tokenString, func(t *jwt.Token) (interface{}, error) {
		if _, ok := t.Method.(*jwt.SigningMethodHMAC); !ok { return nil, fmt.Errorf("unexpected signing method") }
		return s.secret, nil
	})
	if err != nil { return nil, err }
	claims, ok := token.Claims.(jwt.MapClaims)
	if !ok || !token.Valid { return nil, fmt.Errorf("invalid token") }
	userID, _ := uuid.Parse(claims["sub"].(string))
	return &models.User{ID: userID, Email: claims["email"].(string), Name: claims["name"].(string), Role: claims["role"].(string)}, nil
}
