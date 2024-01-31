use arboard::Clipboard;
use device_query::{DeviceQuery, DeviceState, Keycode};
use rodio::{Decoder, OutputStream, Sink};
use std::collections::HashMap;
use std::sync::mpsc;

use std::{io::Cursor, thread, thread::sleep, time::Duration};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum CMD {
    // ShowPlayers
    ShowPlayers,
    // TeleportToPlayer
    MeToPlayer,
    // TeleportToMe
    PlayerToMe,
}

struct AudioPlayer {
    cmd_sender: mpsc::Sender<(CMD, mpsc::Sender<()>)>,
}

impl AudioPlayer {
    fn new() -> Self {
        let (cmd_sender, cmd_receiver) = mpsc::channel::<(CMD, mpsc::Sender<()>)>();
        thread::spawn(move || {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let mut sounds = HashMap::new();

            // Load sounds
            sounds.insert(
                CMD::ShowPlayers,
                include_bytes!("../audios/ShowPlayers.mp3").to_vec(),
            );
            sounds.insert(
                CMD::MeToPlayer,
                include_bytes!("../audios/TeleportToPlayer.mp3").to_vec(),
            );
            sounds.insert(
                CMD::PlayerToMe,
                include_bytes!("../audios/TeleportToMe.mp3").to_vec(),
            );

            for (cmd, done_sender) in cmd_receiver {
                if let Some(sound_data) = sounds.get(&cmd) {
                    let cursor = Cursor::new(sound_data.clone());
                    let source = Decoder::new(cursor).unwrap();
                    let sink = Sink::try_new(&stream_handle).unwrap();
                    sink.append(source);
                    sink.sleep_until_end();
                    done_sender.send(()).unwrap();
                }
            }
        });

        AudioPlayer { cmd_sender }
    }

    fn play(&self, cmd: CMD) {
        let (done_sender, done_receiver) = mpsc::channel();
        self.cmd_sender.send((cmd, done_sender)).unwrap();
        done_receiver.recv().unwrap(); // Wait for the sound to finish playing
    }
}

fn match_cmd(keys: Vec<Keycode>) -> Option<CMD> {
    if keys.len() != 2 {
        return None;
    }
    println!("keys:\t{:?}", keys);
    if keys[1] == Keycode::BackSlash {
        match keys[0] {
            Keycode::S => Some(CMD::ShowPlayers),
            Keycode::P => Some(CMD::MeToPlayer),
            Keycode::M => Some(CMD::PlayerToMe),
            _ => None,
        }
    } else {
        None
    }
}

fn get_player_id(text: String) -> i32 {
    // 判断是否为一个 长度为9 的数字 如：128991414
    if text.len() == 9 {
        if text.parse::<i32>().is_ok() {
            return text.parse::<i32>().unwrap();
        }
    }
    // print!("剪切板内容并不是一个player id\t{}", text);
    return 0;
}

fn update_clipboard(clipboard: &mut Clipboard, command: &str) {
    clipboard.set_text(command).unwrap();
    println!(
        "CMD to your clipboard:\t\t\t\t {} -> {}",
        '\u{1F4CB}', command
    );
}

fn main() {
    let mut clipboard: Clipboard = Clipboard::new().unwrap();
    let device_state = DeviceState::new();
    let mut cmd_str = "".to_string();
    let audio_player = AudioPlayer::new();
    println!(
        "开始监听剪切板和键盘, 按下 Ctrl + C 退出\n\
    1. `\\` + `s`\t\t\tShowPlayers\n\
    2. 复制9位id + `\\` + `p`\tTeleportToPlayer\n\
    3. 复制9位id + `\\` + `m`\tTeleportToMe\n"
    );

    loop {
        cmd_str.clear();
        sleep(Duration::from_millis(100));
        let keys = device_state.get_keys();
        let cmd_type = match_cmd(keys);
        let clip_txt = clipboard.get_text().unwrap_or("".to_string());
        let player_id = get_player_id(clip_txt);

        // 输出 cmd_type
        if cmd_type.is_some() {
            println!("player_id:\t{}", player_id);
        }
        if let Some(cmd) = cmd_type {
            match cmd {
                CMD::ShowPlayers => {
                    update_clipboard(&mut clipboard, "ShowPlayers");
                    audio_player.play(cmd)
                }
                CMD::MeToPlayer | CMD::PlayerToMe if player_id != 0 => {
                    let cmd_str = match cmd {
                        CMD::MeToPlayer => format!("TeleportToPlayer {}", player_id),
                        CMD::PlayerToMe => format!("TeleportToMe {}", player_id),
                        _ => unreachable!(),
                    };

                    update_clipboard(&mut clipboard, &cmd_str);
                    audio_player.play(cmd)
                }
                _ => {}
            }
        }
    }
}

//
// use rodio::{Decoder, OutputStream, source::Source};
// use std::io::Cursor;
//
// fn main() {
//     // 获得输出音频流和声道句柄
//     let (_stream, stream_handle) = OutputStream::try_default().unwrap();
//     // 这里我们将用到的音乐数据直接嵌入到代码中，作为一个字节数组
//     // 这是通过将文件内容转化为字节数组得到的
//     // 例如：你可以使用 `include_bytes!` 宏来实现
//     let music_data = include_bytes!("/Users/zeke/Downloads/2024-01-31-142741_176972.mp3");
//
//     // 使用 `Cursor` 来包装音乐数据的字节数组
//     let cursor = Cursor::new(music_data.as_ref());
//     // 使用 `rodio` 解码音乐流
//     let source = Decoder::new(cursor).unwrap();
//     // 播放音乐
//     println!("开始播放音乐");
//     stream_handle.play_raw(source.convert_samples()).unwrap();
//     println!("播放音乐结束");
//
//     // 音频是非阻塞的，所以我们需要这样阻塞主线程以继续播放
//     std::thread::sleep(std::time::Duration::from_secs(10));
// }
