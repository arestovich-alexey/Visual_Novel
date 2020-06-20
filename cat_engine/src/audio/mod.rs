mod audio_track;
mod sample;
mod sample_rate;
mod channels;

pub use audio_track::*;
use sample_rate::*;

use channels::ChannelCountConverter;

use cpal::{
    Device,
    DevicesError,
    Devices,
    OutputDevices,
    traits::{
        HostTrait,
        DeviceTrait,
        EventLoopTrait
    },
    UnknownTypeOutputBuffer,
    StreamData,
    StreamId,
    EventLoop,
    Sample,
};

use std::{
    io,
    vec::IntoIter,
    iter::Cycle,
    path::Path,
    sync::{Arc,Mutex,LockResult},
    thread::{Builder,JoinHandle}
};

/// Результат выполнения команды.
/// 
/// The result of an executed command.
#[derive(Debug,PartialEq)]
pub enum AudioCommandResult{
    Ok,
    ThreadClosed,
    TrackError,
}

impl AudioCommandResult{
    /// Паникует, если результат не `Ok`.
    /// 
    /// Panics, if the result isn`t `Ok`.
    pub fn unwrap(self){
        if self!=AudioCommandResult::Ok{
            panic!("{:?}",self)
        }
    }

    /// Паникует и выводит сообщение, если результат не `Ok`.
    /// 
    /// Panics и prints the message, if the result isn`t `Ok`.
    pub fn expect(self,msg:&str){
        if self!=AudioCommandResult::Ok{
            panic!("{} {:?}",msg,self)
        }
    }
}

enum AudioSystemCommand{
    AddTrack(Track<i16>),
    PlayOnce(usize),
    PlayForever(usize),
    Stop,
    SetVolume(f32),
    Close,
}

enum Play{
    None,
    Once(ChannelCountConverter<SampleRateConverter<IntoIter<i16>>>),
    Forever(ChannelCountConverter<SampleRateConverter<Cycle<IntoIter<i16>>>>),
}

unsafe impl std::marker::Sync for AudioSystemCommand{}
unsafe impl std::marker::Send for AudioSystemCommand{}

/// Простой аудио движок.
/// Simple audio engine.
/// 
/// Пока только вывод доступен.
/// 
/// Only output is available now.
/// 
pub struct Audio{
    event_loop:Arc<EventLoop>,
    streams:Arc<Mutex<Vec<StreamId>>>,
    command:std::sync::mpsc::Sender<AudioSystemCommand>,
    thread:Option<JoinHandle<()>>,
}

impl Audio{
    /// For default host and device.
    pub fn new(settings:AudioSettings)->io::Result<Audio>{
        let mut volume=0.5f32;
        let mut tracks:Vec<Track<i16>>=Vec::with_capacity(settings.track_buffer_capacity);
        let channels=Arc::new(Mutex::new(Vec::with_capacity(settings.channels)));

        let c=channels.clone();

        let host=cpal::default_host();
        let event_loop=Arc::new(host.event_loop());
        let el=event_loop.clone();
        // Передача команд от управляющего потока выполняющему
        let (sender,receiver)=std::sync::mpsc::channel::<AudioSystemCommand>();

        let thread_result=Builder::new().name("Audio thread".to_string()).stack_size(2048).spawn(move||{
            let mut play=Play::None;

            let device=host.default_output_device().unwrap();
            let mut format=device.default_output_format().unwrap();

            format.channels=settings.output_type.into_channels();

            let main_stream=event_loop.build_output_stream(&device,&format).expect("stream");

            c.lock().unwrap().push(main_stream.clone());

            event_loop.play_stream(main_stream.clone()).unwrap();
            event_loop.clone().run(move|stream,result|{
                match receiver.try_recv(){
                    Ok(command)=>match command{
                        AudioSystemCommand::AddTrack(new_track)=>{
                            if tracks.len()<tracks.capacity(){
                                tracks.push(new_track)
                            }
                        }
                        AudioSystemCommand::PlayOnce(i)=>{
                            let track_channels=tracks[i].channels();
                            let track=tracks[i].clone().into_iter(format.sample_rate);
                            let track=ChannelCountConverter::new(track,track_channels,format.channels);
                            play=Play::Once(track);
                        }
                        AudioSystemCommand::PlayForever(i)=>{
                            let track_channels=tracks[i].channels();
                            let track=tracks[i].clone().endless_iter(format.sample_rate);
                            let track=ChannelCountConverter::new(track,track_channels,format.channels);
                            play=Play::Forever(track);
                        }
                        AudioSystemCommand::Stop=>{
                            play=Play::None;
                        }
                        AudioSystemCommand::SetVolume(v)=>{
                            volume=v;
                        }
                        AudioSystemCommand::Close=>{
                            panic!("Closing audio thread")
                        },
                    }
                    Err(_)=>{}
                }


                match &mut play{
                    Play::None=>{}

                    Play::Once(track)=>{
                        match result{
                            Ok(data)=>{
                                match data{
                                    StreamData::Output{buffer:UnknownTypeOutputBuffer::I16(mut buffer)}
                                    =>for b in buffer.iter_mut(){
                                        *b=(track.next().unwrap_or(0i16) as f32 * volume) as i16;
                                    }

                                    StreamData::Output{buffer:UnknownTypeOutputBuffer::U16(mut buffer)}
                                    =>for b in buffer.iter_mut(){
                                        let sample=(track.next().unwrap_or(0i16) as f32 * volume) as i16;
                                        *b=sample.to_u16();
                                    }

                                    StreamData::Output{buffer:UnknownTypeOutputBuffer::F32(mut buffer)}
                                    =>for b in buffer.iter_mut(){
                                        let sample=track.next().unwrap_or(0i16) as f32 * volume;
                                        *b=sample.to_f32();
                                    }

                                    _=>{}
                                }
                            }
                            Err(e)=>{
                                eprintln!("an error occurred on stream {:?}: {}",stream,e);
                                return
                            }
                        }
                    }

                    Play::Forever(track)=>{
                        match result{
                            Ok(data)=>{
                                match data{
                                    StreamData::Output{buffer:UnknownTypeOutputBuffer::I16(mut buffer)}
                                    =>for b in buffer.iter_mut(){
                                        *b=(track.next().unwrap_or(0i16) as f32 * volume) as i16;
                                    }

                                    StreamData::Output{buffer:UnknownTypeOutputBuffer::U16(mut buffer)}
                                    =>for b in buffer.iter_mut(){
                                        let sample=(track.next().unwrap_or(0i16) as f32 * volume) as i16;
                                        *b=sample.to_u16();
                                    }

                                    StreamData::Output{buffer:UnknownTypeOutputBuffer::F32(mut buffer)}
                                    =>for b in buffer.iter_mut(){
                                        let sample=track.next().unwrap_or(0i16) as f32 * volume;
                                        *b=sample/(i16::max_value() as f32);
                                    }

                                    _=>{}
                                }
                            }
                            Err(e)=>{
                                eprintln!("an error occurred on stream {:?}: {}",stream,e);
                                return
                            }
                        }
                    }
                }
            });
        });

        let thread=match thread_result{
            Ok(thread)=>thread,
            Err(e)=>return Err(e),
        };

        Ok(Self{
            event_loop:el,
            streams:channels,
            command:sender,
            thread:Some(thread),
        })
    }

    pub fn default_output_device()->Option<Device>{
        cpal::default_host().default_output_device()
    }

    pub fn output_device()->Result<OutputDevices<Devices>,DevicesError>{
        cpal::default_host().output_devices()
    }

    /// Добавляет трек в массив треков, удаляет, если массив переполнен.
    /// 
    /// Adds the track from given path to the track array.
    /// Ignores, if the array is overflown.
    pub fn add_track<P:AsRef<Path>>(&self,path:P)->AudioCommandResult{
        let track=match Track::new(path){
            TrackResult::Ok(track)=>track,
            _=>return AudioCommandResult::TrackError
        };
        match self.command.send(AudioSystemCommand::AddTrack(track)){
            Ok(())=>AudioCommandResult::Ok,
            Err(_)=>AudioCommandResult::ThreadClosed
        }
    }

    /// Запускает трек без повторов.
    /// 
    /// Sets a track to play once.
    pub fn play_once(&self,index:usize)->AudioCommandResult{
        match self.command.send(AudioSystemCommand::PlayOnce(index)){
            Ok(())=>AudioCommandResult::Ok,
            Err(_)=>AudioCommandResult::ThreadClosed
        }
    }

    /// Запускает трек, который постоянно повторяется.
    /// 
    /// Sets a track to play forever.
    pub fn play_forever(&self,index:usize)->AudioCommandResult{
        match self.command.send(AudioSystemCommand::PlayForever(index)){
            Ok(())=>AudioCommandResult::Ok,
            Err(_)=>AudioCommandResult::ThreadClosed
        }
    }

    /// Запускает проигрывание канала.
    /// 
    /// Starts playing the stream.
    pub fn play(self)->AudioCommandResult{
        let stream=match self.streams.lock(){
            LockResult::Ok(streams)=>streams.get(0).unwrap().clone(),
            LockResult::Err(_)=>return AudioCommandResult::ThreadClosed
        };
        self.event_loop.play_stream(stream);
        AudioCommandResult::Ok
    }
    /// Ставит на паузу проигрывание канала.
    /// 
    /// Pauses the stream.
    pub fn pause(&self)->AudioCommandResult{
        let stream=match self.streams.lock(){
            LockResult::Ok(streams)=>streams.get(0).unwrap().clone(),
            LockResult::Err(_)=>return AudioCommandResult::ThreadClosed
        };
        self.event_loop.pause_stream(stream);
        AudioCommandResult::Ok
    }

    /// Останавливает проигрывание путём удаления трека из буфера для вывода.
    /// 
    /// Stops playing by removing track from playing buffer.
    pub fn stop(&self)->AudioCommandResult{
        match self.command.send(AudioSystemCommand::Stop){
            Ok(())=>AudioCommandResult::Ok,
            Err(_)=>AudioCommandResult::ThreadClosed
        }
    }

    /// Устанавливает громкость.
    /// 
    /// Sets the volume.
    pub fn set_volume(&self,volume:f32)->AudioCommandResult{
        match self.command.send(AudioSystemCommand::SetVolume(volume)){
            Ok(())=>AudioCommandResult::Ok,
            Err(_)=>AudioCommandResult::ThreadClosed
        }
    }
}

/// Отправляет команду для остановки и ожидает окончание работы потока.
/// 
/// Sends closing command and then waits for the thread to finish.
impl Drop for Audio{
    fn drop(&mut self){
        let _=self.command.send(AudioSystemCommand::Close);
        if let Some(thread)=self.thread.take(){
            let _=thread.join();
        }
        println!("Dropped");
    }
}


/// Тип аудио вывода.
/// 
/// Audio output type.
#[derive(Clone)]
pub enum AudioOutputType{
    Mono,
    Stereo,
}

impl AudioOutputType{
    pub fn into_channels(self)->u16{
        match self{
            AudioOutputType::Mono=>1u16,
            AudioOutputType::Stereo=>2u16,
        }
    }
}

pub struct AudioSettings{
    pub output_type:AudioOutputType,
    pub channels:usize,
    pub track_buffer_capacity:usize,
}

impl AudioSettings{
    pub fn new()->AudioSettings{
        Self{
            output_type:AudioOutputType::Stereo,
            channels:1,
            track_buffer_capacity:1,
        }
    }
}