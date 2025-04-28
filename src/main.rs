#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::{Spawner};
use embassy_stm32::gpio::{AnyPin, Input, Level, Output, Pull, Speed};
use embassy_stm32::peripherals::PA0;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{Channel, Sender};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

// 定义按钮事件类型
#[derive(Debug, Clone, Copy, Format)]
pub enum ButtonEvent {
    // Button1Pressed,
    Button2Pressed,
    Button3Pressed,
    Button4Pressed,
}

// 定义信道：线程模式原始互斥锁 + 按钮事件类型 + 信道容量为3
static CHANNEL: Channel<ThreadModeRawMutex, ButtonEvent, 3> = Channel::new();

/*
    功能需求
        1. 按下按钮3 LED3亮起 再次按下按钮3 LED3熄灭
        2. 按下按钮4 LED4亮起 再次按下按钮4 LED4熄灭
*/
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("程序开始执行.....");

    // 启动按钮1任务
    // spawner.spawn(button_task(AnyPin::from(p.PF8), ButtonEvent::Button1Pressed)).unwrap();

    // 启动按钮2任务 不知道为什么按了没有效果
    spawner.spawn(button_task(AnyPin::from(p.PF8), ButtonEvent::Button2Pressed)).unwrap();

    // 启动按钮3任务
    spawner.spawn(button_task(AnyPin::from(p.PF10), ButtonEvent::Button3Pressed)).unwrap();

    // 启动按钮4任务
    spawner.spawn(button_task(AnyPin::from(p.PF11), ButtonEvent::Button4Pressed)).unwrap();

    // 启动LED控制任务
    spawner.spawn(led_task(AnyPin::from(p.PA0), AnyPin::from(p.PA1), AnyPin::from(p.PA8))).unwrap();
}

// 按钮任务：检测按钮按下并发送事件到信道
#[embassy_executor::task(pool_size = 4)]
async fn button_task(button: AnyPin, event: ButtonEvent) {
    let button = Input::new(button, Pull::Down);
    let mut button_pressed = false;

    loop {
        if button.is_high() {
            if !button_pressed {
                button_pressed = true;
                info!("按钮事件发送: {:?}", event);
                CHANNEL.send(event).await; // 发送按钮事件到信道
            }
        } else {
            button_pressed = false;
        }
        Timer::after(Duration::from_millis(10)).await; // 防抖
    }
}

// LED控制任务：从信道接收事件并控制对应的LED
#[embassy_executor::task(pool_size = 4)]
async fn led_task(led2: AnyPin, led3: AnyPin, led4: AnyPin) {
    let mut led2 = Output::new(led2, Level::High, Speed::Low); // 初始化LED2
    let mut led3 = Output::new(led3, Level::High, Speed::Low); // 初始化LED3
    let mut led4 = Output::new(led4, Level::High, Speed::Low); // 初始化LED4

    loop {
        match CHANNEL.receive().await {
            ButtonEvent::Button2Pressed => {
                if led2.is_set_high() {
                    info!("LED2关闭");
                    led2.set_low();
                } else {
                    info!("LED2打开");
                    led2.set_high();
                }
            }
            ButtonEvent::Button3Pressed => {
                if led3.is_set_high() {
                    info!("LED3关闭");
                    led3.set_low();
                } else {
                    info!("LED3打开");
                    led3.set_high();
                }
            }
            ButtonEvent::Button4Pressed => {
                if led4.is_set_high() {
                    info!("LED4关闭");
                    led4.set_low();
                } else {
                    info!("LED4打开");
                    led4.set_high();
                }
            }
        }
    }
}