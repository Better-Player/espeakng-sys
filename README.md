# eSpeak NG bindings for Rust
![Crates.io](https://img.shields.io/crates/v/espeakng-sys?style=flat)  
FFI bindings to the C library eSpeak NG for Rust

Current eSpeak NG version: 1.51

## Dependencies
- [eSpeak NG](https://github.com/espeak-ng/espeak-ng/blob/master/docs/building.md)

## Example
This example shows how you can convert a &str to a Vec<i16>
```rs
#![allow(non_upper_case_globals)]

use espeakng_sys::*;
use std::os::raw::{c_char, c_short, c_int};
use std::ffi::{c_void, CString};
use std::cell::Cell;
use lazy_static::lazy_static;
use std::sync::{Mutex, MutexGuard};

/// The name of the voice to use
const VOICE_NAME: &str = "English";
/// The length in mS of sound buffers passed to the SynthCallback function.
const BUFF_LEN: i32 = 500;
/// Options to set for espeak-ng
const OPTIONS: i32 = 0;

lazy_static! {
    /// The complete audio provided by the callback
    static ref AUDIO_RETURN: Mutex<Cell<Vec<i16>>> = Mutex::new(Cell::new(Vec::default()));

    /// Audio buffer for use in the callback
    static ref AUDIO_BUFFER: Mutex<Cell<Vec<i16>>> = Mutex::new(Cell::new(Vec::default()));
}

/// Spoken speech
pub struct Spoken {
    /// The audio data
    pub wav:            Vec<i16>,
    /// The sample rate of the audio
    pub sample_rate:    i32
}

/// Perform Text-To-Speech
pub fn speak(text: &str) -> Spoken {
    let output: espeak_AUDIO_OUTPUT = espeak_AUDIO_OUTPUT_AUDIO_OUTPUT_RETRIEVAL;

    AUDIO_RETURN.plock().set(Vec::default());
    AUDIO_BUFFER.plock().set(Vec::default());

    // The directory which contains the espeak-ng-data directory, or NULL for the default location.
    let path: *const c_char = std::ptr::null();
    let voice_name_cstr = CString::new(VOICE_NAME).expect("Failed to convert &str to CString");
    let voice_name = voice_name_cstr.as_ptr();

    // Returns: sample rate in Hz, or -1 (EE_INTERNAL_ERROR).
    let sample_rate = unsafe { espeak_Initialize(output, BUFF_LEN, path, OPTIONS) };

    unsafe {
        espeak_SetVoiceByName(voice_name as *const c_char);
        espeak_SetSynthCallback(Some(synth_callback))
    }

    let text_cstr = CString::new(text).expect("Failed to convert &str to CString");

    let position = 0u32;
    let position_type: espeak_POSITION_TYPE = 0;
    let end_position = 0u32;
    let flags = espeakCHARS_AUTO;
    let identifier = std::ptr::null_mut();
    let user_data = std::ptr::null_mut();

    unsafe { espeak_Synth(text_cstr.as_ptr() as *const c_void, BUFF_LEN as size_t, position, position_type, end_position, flags, identifier, user_data); }

    // Wait for the speaking to complete
    match unsafe { espeak_Synchronize() } {
        espeak_ERROR_EE_OK => {},
        espeak_ERROR_EE_INTERNAL_ERROR => {
            todo!()
        }
        _ => unreachable!()
    }

    let result = AUDIO_RETURN.plock().take();

    Spoken {
        wav: result,
        sample_rate
    }
}

/// int SynthCallback(short *wav, int numsamples, espeak_EVENT *events);
///
/// wav:  is the speech sound data which has been produced.
/// NULL indicates that the synthesis has been completed.
///
/// numsamples: is the number of entries in wav.  This number may vary, may be less than
/// the value implied by the buflength parameter given in espeak_Initialize, and may
/// sometimes be zero (which does NOT indicate end of synthesis).
///
/// events: an array of espeak_EVENT items which indicate word and sentence events, and
/// also the occurance if <mark> and <audio> elements within the text.  The list of
/// events is terminated by an event of type = 0.
///
/// Callback returns: 0=continue synthesis,  1=abort synthesis.
unsafe extern "C" fn synth_callback(wav: *mut c_short, sample_count: c_int, events: *mut espeak_EVENT) -> c_int {

    // Calculate the length of the events array
    let mut events_copy = events.clone();
    let mut elem_count = 0;
    while (*events_copy).type_ != espeak_EVENT_TYPE_espeakEVENT_LIST_TERMINATED {
        elem_count += 1;
        events_copy = events_copy.add(1);
    }

    // Turn the event array into a Vec.
    // We must clone from the slice, as the provided array's memory is managed by C
    let event_slice = std::slice::from_raw_parts_mut(events, elem_count);
    let event_vec = event_slice.into_iter()
        .map(|f| f.clone())
        .collect::<Vec<espeak_EVENT>>();

    // Turn the audio wav data array into a Vec.
    // We must clone from the slice, as the provided array's memory is managed by C
    let wav_slice = std::slice::from_raw_parts_mut(wav, sample_count as usize);
    let mut wav_vec = wav_slice.into_iter()
        .map(|f| f.clone() as i16)
        .collect::<Vec<i16>>();

    // Determine if this is the end of the synth
    let mut is_end = false;
    for event in event_vec {
        if event.type_.eq(&espeak_EVENT_TYPE_espeakEVENT_MSG_TERMINATED) {
            is_end = true;
        }
    }

    // If this is the end, we want to set the AUDIO_RETURN
    // Else we want to append to the AUDIO_BUFFER
    if is_end {
        AUDIO_RETURN.plock().set(AUDIO_BUFFER.plock().take());
    } else {
        let mut curr_data = AUDIO_BUFFER.plock().take();
        curr_data.append(&mut wav_vec);
        AUDIO_BUFFER.plock().set(curr_data);
    }

    0
}

trait PoisonlessLock<T> {
    fn plock(&self) -> MutexGuard<T>;
}

impl<T> PoisonlessLock<T> for Mutex<T> {
    fn plock(&self) -> MutexGuard<T> {
        match self.lock() {
            Ok(l) => l,
            Err(e) => e.into_inner()
        }
    }

```

## Crate features

The `clang-runtime` features enables the `bindgen`'s `runtime` feature, which in turn is forwarded
to the `runtime` feature in the [`clang-sys`](https://github.com/KyleMayes/clang-sys#dependencies) crate. 
This enables dynamic linking of `libclang` at runtime. This
feature flag is present to provide compatibility with other crates that require the `bindgen/runtime` feature.
If you get the error ```'a `libclang` shared library is not loaded on this thread'```, try
enabling the `clang-runtime` feature.

## License
`espeakng-sys` is dual licensed under the Apache-2.0 and MIT license, at your discretion.
Do note though, `espeak-ng` itself is licensed under the [GPL](http://www.gnu.org/licenses)

