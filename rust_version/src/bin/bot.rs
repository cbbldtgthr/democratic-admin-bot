use rust_version::AddListing;
use std::error::Error;
use teloxide::types::User;
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};
use teloxide::{
    payloads::SendMessageSetters,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup, ReplyMarkup,
    },
};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default, Debug)]
pub enum State {
    #[default]
    Start,
    Menu,

    BuyAndSell,

    SellSomething {
        photos: Vec<String>,
    },
    AddDescription {
        photos: Vec<String>,
    },
    Confirm {
        photos: Vec<String>,
        description: String,
    },
    AnonPoll {
        question: String,
        selections: Vec<String>,
    },
}

fn keyboard(items: Vec<&str>) -> KeyboardMarkup {
    KeyboardMarkup::default()
        .resize_keyboard()
        .append_row(items.into_iter().map(|name| KeyboardButton::new(name)))
}

#[derive(Clone, Default)]
pub enum StateBuyAndSell {
    #[default]
    Start,
    ReceiveFullName,
    ReceiveAge {
        full_name: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::info!("Starting buttons bot...");

    let bot = Bot::from_env();

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .enter_dialogue::<Message, InMemStorage<State>, State>()
                .branch(dptree::case![State::Start].endpoint(start))
                .branch(dptree::case![State::Menu].endpoint(menu))
                .branch(dptree::case![State::BuyAndSell].endpoint(buy_and_sell))
                .branch(dptree::case![State::SellSomething { photos }].endpoint(sell_something))
                .branch(dptree::case![State::AddDescription { photos }].endpoint(added_description))
                .branch(
                    dptree::case![State::Confirm {
                        photos,
                        description
                    }]
                    .endpoint(confirm),
                ),
        )
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}

async fn goto_menu(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Main menu")
        .reply_markup(keyboard(vec![
            "Buy-and-Sell",
            "Anonymous poll",
            "Invite someone",
            "Help",
            "Main menu",
        ]))
        .await?;
    dialogue.update(State::Menu).await?;
    Ok(())
}

async fn default(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some("Main menu") => {}
        Some(_) => {
            bot.send_message(msg.chat.id, "Unknown option").await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Did not expect this...")
                .await?;
        }
    }
    goto_menu(bot, dialogue, msg).await?;
    Ok(())
}
async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    println!("{:?}", msg);
    goto_menu(bot, dialogue, msg).await?;
    Ok(())
}

async fn menu(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some("Buy-and-Sell") => {
            bot.send_message(msg.chat.id, "Would you like to:")
                .reply_markup(keyboard(vec![
                    "List something new",
                    "List my items",
                    "Search listings",
                    "Main menu",
                ]))
                .await?;
            dialogue.update(State::BuyAndSell).await?;
        }
        Some("Help") => {
            bot.send_message(msg.chat.id, "Please read more at welgevonden.dev")
                .await?;
            dialogue.update(State::Menu).await?;
        }
        _ => {
            default(bot, dialogue, msg).await?;
        }
    }
    Ok(())
}
async fn goto_sell_something(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(
                msg.chat.id,
                "Upload some photos. You can add more than one photo for a single listing. Click 'Done' once finished",
            )
            .reply_markup(keyboard(vec!["Done uploading", "Cancel"]))
            .await?;
    dialogue
        .update(State::SellSomething { photos: vec![] })
        .await?;
    Ok(())
}

async fn buy_and_sell(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some("List something new") => {
            goto_sell_something(bot, dialogue, msg).await?;
        }
        Some("List my items") => {
            bot.send_message(msg.chat.id, "Tutoring")
                .reply_markup(make_delete_keyboard())
                .await?;
            bot.send_message(msg.chat.id, "Cat sitting")
                .reply_markup(make_delete_keyboard())
                .await?;
            dialogue.update(State::BuyAndSell).await?;
        }
        Some("Search listings") => {
            bot.send_message(
                msg.chat.id,
                "You can see and search all active listings at http://welgevonden.dev/listings",
            )
            .await?;
            goto_menu(bot, dialogue, msg).await?
        }
        _ => {
            default(bot, dialogue, msg).await?;
        }
    }

    Ok(())
}

async fn goto_edit_description(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Give a description. Remeber to include the price.",
    )
    .reply_markup(ReplyMarkup::kb_remove())
    .await?;
    let current_state = dialogue.get().await?.unwrap();
    if let State::SellSomething { photos } = current_state {
        dialogue.update(State::AddDescription { photos }).await?;
    } else if let State::AddDescription { photos } = current_state {
        dialogue.update(State::AddDescription { photos }).await?;
    }
    Ok(())
}

async fn sell_something(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some("Cancel") => {
            bot.send_message(msg.chat.id, "Cancelled").await?;
            goto_menu(bot, dialogue, msg).await?;
        }
        Some("Done uploading") => goto_edit_description(bot, dialogue, msg).await?,
        _ => match msg.photo() {
            Some(photo) => {
                let mut current_state = dialogue.get().await?.unwrap();
                println!("{:?}", photo);
                println!("{:?}", current_state);
                if let State::SellSomething { photos } = &mut current_state {
                    photos.push("hallo".into()); // Append the new photo
                }
                dialogue.update(current_state).await?;
            }
            _ => {
                default(bot, dialogue, msg).await?;
            }
        },
    }
    Ok(())
}

async fn added_description(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some("Cancel") => {
            bot.send_message(msg.chat.id, "Cancelled").await?;
            goto_menu(bot, dialogue, msg).await?;
        }
        Some(text) => {
            let current_state = dialogue.get().await?.unwrap();
            if let State::AddDescription { photos } = current_state {
                dialogue
                    .update(State::Confirm {
                        photos,
                        description: text.into(),
                    })
                    .await?;

                bot.send_message(msg.chat.id, "Would you like to publish this?")
                    .reply_markup(keyboard(vec![
                        "Confirm",
                        "Edit description",
                        "Start again",
                        "Cancel",
                    ]))
                    .await?;
            }
        }
        _ => {
            default(bot, dialogue, msg).await?;
        }
    }
    Ok(())
}

async fn confirm(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some("Start again") => goto_sell_something(bot, dialogue, msg).await?,
        Some("Edit description") => goto_edit_description(bot, dialogue, msg).await?,
        Some("Cancel") => {
            bot.send_message(msg.chat.id, "Cancelled").await?;
            goto_menu(bot, dialogue, msg).await?;
        }
        Some("Confirm") => {
            let current_state = dialogue.get().await?.unwrap();
            if let State::Confirm {
                photos,
                description,
            } = current_state
            {
                if let Some(user) = &msg.from {
                    let UserId(user_telegram_id) = user.id;
                    println!("{}, {}", photos.join("/"), description);
                    reqwest::Client::new()
                        .post("http://127.0.0.1:3000/listing")
                        .json(&AddListing {
                            user_telegram_id,
                            description,
                        })
                        .send()
                        .await?;
                } else {
                    println!("No user found in {:?}", msg);
                }
            }
            bot.send_message(msg.chat.id, "Published! Check the Buy-and-sell channel")
                .await?;
            goto_menu(bot, dialogue, msg).await?
        }
        _ => {
            default(bot, dialogue, msg).await?;
        }
    }
    Ok(())
}

/// Parse the text wrote on Telegram and check if that text is a valid command
/// or not, then match the command. If the command is `/start` it writes a
/// markup with the `InlineKeyboardMarkup`.

fn make_delete_keyboard() -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    let debian_versions = ["Remove"];

    for versions in debian_versions.chunks(3) {
        let row = versions
            .iter()
            .map(|&version| InlineKeyboardButton::callback(version.to_owned(), version.to_owned()))
            .collect();

        keyboard.push(row);
    }

    InlineKeyboardMarkup::new(keyboard)
}

/// When it receives a callback from a button it edits the message with all
/// those buttons writing a text with the selected Debian version.
///
/// **IMPORTANT**: do not send privacy-sensitive data this way!!!
/// Anyone can read data stored in the callback button.
async fn callback_handler(bot: Bot, q: CallbackQuery) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(ref version) = q.data {
        // Tell telegram that we've seen this query, to remove 🕑 icons from the
        // clients. You could also use `answer_callback_query`'s optional
        // parameters to tweak what happens on the client side.
        bot.answer_callback_query(&q.id).await?;

        // Edit text of the message to which the buttons were attached
        if let Some(message) = q.regular_message() {
            if let Some(content) = message.text() {
                let text = format!("Removed: {}", content);
                bot.edit_message_text(message.chat.id, message.id, text)
                    .await?;
            }
        } else if let Some(id) = q.inline_message_id {
            bot.edit_message_text_inline(id, "error").await?;
        }

        log::info!("You chose: {}", version);
    }

    Ok(())
}
