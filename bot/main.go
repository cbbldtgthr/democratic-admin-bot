package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"

	"github.com/gin-gonic/gin"
)

type Update struct {
	Message *Message `json:"message"`
}

type Message struct {
	Chat *Chat  `json:"chat"`
	From *User  `json:"from"`
	Text string `json:"text"`
}

type Chat struct {
	ID   int64  `json:"id"`
	Type string `json:"type"`
}

type User struct {
	ID       int64  `json:"id"`
	Username string `json:"username"`
}

type PollResponse struct {
	PollID string `json:"poll_id"`
}

const (
	telegramAPI = "https://api.telegram.org/bot"
)

var token string

func main() {
	if err := loadToken(); err != nil {
		log.Fatalf("Failed to load bot token: %v", err)
	}

	r := gin.Default()
	r.POST("/webhook", webhookHandler)
	log.Fatal(r.Run(":8080"))
}

func loadToken() error {
	file, err := os.Open(".env.json")
	if err != nil {
		return fmt.Errorf("could not open .env.json: %w", err)
	}
	defer file.Close()

	var config struct {
		TelegramToken string `json:"telegram_token"`
	}
	if err := json.NewDecoder(file).Decode(&config); err != nil {
		return fmt.Errorf("could not decode .env.json: %w", err)
	}

	token = config.TelegramToken
	return nil
}

func webhookHandler(c *gin.Context) {
	var update Update
	if err := c.ShouldBindJSON(&update); err != nil {
		log.Printf("Error parsing update: %v", err)
		c.Status(http.StatusBadRequest)
		return
	}

	if update.Message == nil || update.Message.Chat.Type != "group" {
		c.Status(http.StatusOK)
		return
	}

	text := update.Message.Text
	if len(text) > 6 && text[:6] == "/kick " {
		targetUser := text[6:]
		if targetUser[0] != '@' {
			log.Println("Invalid user format. Should start with '@'.")
			c.Status(http.StatusOK)
			return
		}

		if err := sendKickPoll(update.Message.Chat.ID, targetUser); err != nil {
			log.Printf("Failed to send poll: %v", err)
			c.Status(http.StatusInternalServerError)
			return
		}
	}

	c.Status(http.StatusOK)
}

func sendKickPoll(chatID int64, targetUser string) error {
	pollQuestion := fmt.Sprintf("Should we kick %s from the group?", targetUser)
	pollOptions := []string{"Yes", "No"}

	body, err := json.Marshal(map[string]interface{}{
		"chat_id":      chatID,
		"question":     pollQuestion,
		"options":      pollOptions,
		"is_anonymous": false,
	})
	if err != nil {
		return fmt.Errorf("failed to marshal poll data: %v", err)
	}

	reqBody := bytes.NewBuffer(body)
	resp, err := http.Post(fmt.Sprintf("%s%s/sendPoll", telegramAPI, token), "application/json", reqBody)
	if err != nil {
		return fmt.Errorf("failed to send poll request: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("non-200 response from Telegram API: %s", resp.Status)
	}

	log.Println("Poll sent successfully.")
	return nil
}
