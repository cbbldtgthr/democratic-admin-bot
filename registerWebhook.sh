#!/bin/bash

# This script registers the serverless POST endpoint as the webhook for the
# Telegram bot. I.e. this is the enpoint that the bot will call when it recieves
# a message

# Source env file 
. .env

# Register enpoint
curl https://api.telegram.org/bot${TELEGRAM_TOKEN}/setWebhook?url=${WEBHOOK_ENDPOINT}

# Confirm registration
curl https://api.telegram.org/bot${TELEGRAM_TOKEN}/getWebhookInfo
