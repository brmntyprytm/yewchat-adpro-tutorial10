use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
}
impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
    let submit = ctx.link().callback(|_| Msg::SubmitMessage);

    html! {
        <div class="flex w-screen h-screen bg-gray-900 text-white">
            <div class="flex-none w-1/4 h-full bg-gray-800 overflow-y-auto">
                <div class="text-xl p-3 border-b border-gray-700">{"Users"}</div>
                {
                    self.users.clone().iter().map(|u| {
                        html!{
                            <div class="flex items-center m-3 bg-gray-700 rounded-lg p-2">
                                <img class="w-12 h-12 rounded-full" src={u.avatar.clone()} alt="avatar"/>
                                <div class="ml-3 text-sm">{u.name.clone()}</div>
                            </div>
                        }
                    }).collect::<Html>()
                }
            </div>
            <div class="flex-grow flex flex-col">
                <div class="flex-grow overflow-y-auto px-6 py-4">
                    {
                        self.messages.iter().map(|m| {
                            let user = self.users.iter().find(|u| u.name == m.from).unwrap();
                            html!{
                                <div class="flex items-start mb-4">
                                    <img class="w-10 h-10 rounded-full mr-4" src={user.avatar.clone()} alt="avatar"/>
                                    <div class="bg-gray-700 p-4 rounded-lg">
                                        <div class="text-sm">{m.from.clone()}</div>
                                        <div class="text-gray-200 mt-1">
                                            {
                                                if m.message.ends_with(".gif") {
                                                    html!{<img src={m.message.clone()} alt="gif" class="max-w-xs"/>}
                                                } else {
                                                    html!{<p>{m.message.clone()}</p>}
                                                }
                                            }
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
                <div class="w-full h-14 flex items-center justify-between bg-gray-800 border-t border-gray-700">
                    <div class="flex items-center w-full">
                        <input ref={self.chat_input.clone()} type="text" placeholder="Message" class="py-2 pl-4 pr-10 mx-3 bg-gray-700 rounded-full outline-none focus:ring-2 focus:ring-green-500 focus:border-transparent text-white" name="message" required=true />
                        <button onclick={submit} class="p-3 shadow-sm bg-green-600 w-10 h-10 rounded-full flex justify-center items-center text-white">
                            <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="fill-current w-6 h-6">
                                <path d="M0 0h24v24H0z" fill="none"></path>
                                <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}





    
}