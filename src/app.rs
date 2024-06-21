use chrono::{
    DateTime, Days, Duration, FixedOffset, Local, NaiveDateTime, NaiveTime, ParseResult, TimeZone,
};
use gloo_timers::callback::Timeout;
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;
use yew::{html, Component, Context, Html, InputEvent, TargetCast};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    fn invoke(cmd: &str);
}

const FORCE_SHUTDOWN_COUNTER: u8 = 3;

pub struct App {
    shutdown_time: Option<DateTime<FixedOffset>>,
    timeout_handle: Option<Timeout>,
    force_shutdown_counter: u8,
    remain_second_for_shutdown: u32,
    is_countdown_active: bool,
}

pub enum Msg {
    UpdateShutdownTime(String),
    SetShutdownTimer,
    Shutdown(bool),
    PredefinedShutdownTime(i64),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            shutdown_time: Some(Local::now().fixed_offset()),
            timeout_handle: None,
            force_shutdown_counter: FORCE_SHUTDOWN_COUNTER,
            remain_second_for_shutdown: 0,
            is_countdown_active: true,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateShutdownTime(time) => {
                match self.parse_shutdown_time(&time) {
                    Ok(parsed_time) => self.shutdown_time = Some(parsed_time),
                    Err(_) => {}
                }
                true
            }
            Msg::SetShutdownTimer => {
                if let Some(handle) = self.timeout_handle.take() {
                    handle.cancel();
                }
                if self.shutdown_time.is_some() {
                    let current_time = chrono::Local::now();
                    let mut new_time = current_time
                        .with_time(self.shutdown_time.unwrap().time())
                        .unwrap();

                    if current_time > new_time {
                        new_time = new_time.checked_add_days(Days::new(1)).unwrap();
                    }

                    let duration = new_time.signed_duration_since(chrono::Local::now());

                    if duration.num_seconds() > 0 {
                        let link = _ctx.link().clone();
                        let handle =
                            Timeout::new(duration.num_seconds() as u32 * 1000, move || {
                                link.send_message(Msg::Shutdown(true));
                            });

                        self.set_shutdown_time(Some(handle))
                    }
                }

                true
            }
            Msg::Shutdown(is_from_system) => {
                if self.force_shutdown_counter == 0 || is_from_system {
                    invoke("shutdown_pc");
                } else {
                    self.force_shutdown_counter -= 1;
                }

                true
            }
            Msg::PredefinedShutdownTime(predefined_time) => {
                let postpone_time = Local::now() + Duration::minutes(predefined_time);
                self.shutdown_time = Some(postpone_time.fixed_offset());

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
        <div class="flex min-h-screen flex-col items-center justify-center">
            <img src="public/samurai-logo.png" alt="Logo" class="mx-auto mb-4 h-32 w-32 logo" />

            // <div class="mt-2 w-full max-w-sm px-14">
            //     <div class="w-full bg-gray-200 rounded-full h-2.5 dark:bg-gray-700">
            //         <div class="bg-red-600 h-2.5 rounded-full dark:bg-red-500" style="width: 0%"></div>
            //     </div>
            //     <div class="flex justify-center mt-1">
            //         <span class="text-sm font-medium text-blue-700 dark:text-white">{"Remaining Minute: -"}</span>
            //     </div>
            // </div>

            <div class="flex items-center mt-4">
                <input type="time" placeholder="hrs:mins" class="mr-2 w-full rounded-md bg-neutral-900 px-6 py-2 text-white" min="00:00" max="23:59" value={
                    if (*self).shutdown_time.is_some(){
                        ((*self).shutdown_time).unwrap().format("%H:%M").to_string()
                    }else{
                        String::from("00:00")
                    }
                }
                oninput={ctx.link().callback(|e: InputEvent| {
                    let input: HtmlInputElement = e.target_unchecked_into();
                    Msg::UpdateShutdownTime(input.value().trim().to_string())
                })} />
                <button onclick={ctx.link().callback(|_| Msg::SetShutdownTimer)} class="rounded-md bg-neutral-900 px-4 py-2 text-white">{"Set"}</button>
            </div>

            <div class="flex items-center mt-3">
                <span onclick={ctx.link().callback(|_| Msg::PredefinedShutdownTime(10))} class="left-0 top-0 mr-2 rounded-lg bg-green-500 px-2 py-1 text-xs text-white">{"10 Min"}</span>
                <span onclick={ctx.link().callback(|_| Msg::PredefinedShutdownTime(20))} class="left-0 top-0 mr-2 rounded-lg bg-green-600 px-2 py-1 text-xs text-white">{"20 Min"}</span>
                <span onclick={ctx.link().callback(|_| Msg::PredefinedShutdownTime(30))} class="left-0 top-0 rounded-lg bg-green-700 px-2 py-1 text-xs text-white">{"30 Min"}</span>
            </div>

            <div class="mt-2">
                <button onclick={ctx.link().callback(|_| Msg::Shutdown(false))} class="rounded-md bg-orange-600 flex px-4 py-2 text-white mt-2">{"Shutdown Now"} {" ("}{self.force_shutdown_counter}{")"}</button>
            </div>

        </div>
        }
    }
}

impl App {
    fn parse_shutdown_time(
        &mut self,
        date_time_string: &String,
    ) -> ParseResult<DateTime<FixedOffset>> {
        let parsed_time =
            NaiveTime::parse_from_str(date_time_string, "%H:%M").expect("Failed to parse time");

        let today = Local::now().naive_local();
        let parsed_datetime = NaiveDateTime::new(today.into(), parsed_time);

        let local_datetime: DateTime<Local> = Local.from_local_datetime(&parsed_datetime).unwrap();

        let fixed_offset_datetime: DateTime<FixedOffset> =
            local_datetime.with_timezone(local_datetime.offset());

        ParseResult::Ok(fixed_offset_datetime)
    }

    fn set_shutdown_time(&mut self, timeout: Option<Timeout>) {
        self.is_countdown_active = true;
        self.timeout_handle = timeout;
    }

    fn reset(&mut self) {
        self.is_countdown_active = false;
        self.timeout_handle = None;
    }
}
