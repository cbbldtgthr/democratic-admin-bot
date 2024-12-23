export TELOXIDE_TOKEN=$(cat ../.env.json | jq ".telegram_token" -r)

