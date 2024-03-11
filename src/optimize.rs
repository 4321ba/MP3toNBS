
use crate::note;
use crate::wave;
use crate::debug;

use babycat::{Signal, Waveform};





use std::cmp::max;
fn calculate_distance(song_part: &[Vec<f32>], sample: &Vec<Vec<f32>>, dist: &dyn Fn(f32, f32) -> f32, sample_volume: f32) -> f32 {
    let bigger_width = max(song_part.len(), sample.len());
    assert!(song_part[0].len() == sample[0].len()); // though dfferent vectors could still be different sizes
    let mut distance = 0.0_f32;
    for x in 0..bigger_width {
        for y in 0..song_part[0].len() {
            let song_part_val = match song_part.get(x) { Some(v) => v[y], None => 0.0 };
            let sample_val = match sample.get(x) { Some(v) => v[y], None => 0.0 };
            distance += dist(song_part_val, sample_val * sample_volume);
        }
    }
    distance
}

fn calculate_assymetric_distance(song_part: &[Vec<f32>], sample: &Vec<Vec<f32>>, sample_volume: f32) -> f32 {
    calculate_distance(song_part, sample, &|sp, sa| if sp >= sa {0.0} else {(sp-sa) * (sp-sa)}, sample_volume)
}


use debug::debug_save_as_image;
use debug::debug_save_as_wav;
use note::Note;

pub fn test_distances_for_instruments(waveform: &Waveform, cache: &note::CachedInstruments) {
    let mut test_found_notes: Vec<note::Note> = Vec::new();

    let fft_size = 4096;
    let samples = waveform.to_interleaved_samples();
    let spectrogram = wave::create_spectrum(samples, waveform.frame_rate_hz(), fft_size, 1024);
    let spectrogram_2dvec = wave::spectrum_to_2d_vec(&spectrogram);
    let song_part = &spectrogram_2dvec[0..40];
    debug_save_as_image(song_part, "song_part.png");
    for instr_idx in 0..note::INSTRUMENT_COUNT {
        print!("\ninstr idx: {}\n", instr_idx);
        for pitch in 0..note::PITCH_COUNT {
            let sample_2dvec = &cache.spectrograms[instr_idx][pitch];
            debug_save_as_image(&wave::subtract_2d_vecs(song_part, &sample_2dvec), &format!("{instr_idx}_pitch{pitch:02}.png"));

            let TEMP_volume = 0.5; // TODO
            let diff = calculate_assymetric_distance(song_part, &sample_2dvec, TEMP_volume); // TODO
            
            let silence = [vec![0.0; sample_2dvec[0].len()]; 1];
            let compensation = calculate_assymetric_distance(&silence, &sample_2dvec, TEMP_volume); // TODO
            let compensated_val = diff / compensation;
            println!("{pitch:02}: {diff:.5}, comp:{compensation:.5}, compensated: {compensated_val:.5}");

            if compensated_val < 0.015 {
                test_found_notes.push(Note {instrument_id: instr_idx, pitch, volume: TEMP_volume});
                println!("Added this note!");
            }
        }
    }


    let found_wf = note::add_notes_together(&test_found_notes, cache, 1.0);
    debug_save_as_wav(&found_wf, "test_found_notes.wav");
}













use argmin::core::{CostFunction, Error, Executor};
use argmin::solver::particleswarm::ParticleSwarm;
struct Opti<'a> {
    cache: &'a note::CachedInstruments,
    multiplier: f32,
}
impl CostFunction for Opti<'_> {
    type Param = note::NoteStateSpace;
    type Output = f32;
    fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {

        //let wf = note::add_notes_together_statespace(param, self.cache, self.multiplier);
        

        //Ok(0.0)

        Ok((param[0]-0.34) *(param[0]-0.34)+ (param[1]-0.36) *(param[1]-0.36))
    }
}

pub fn optimize(cache: &note::CachedInstruments) {

    let cost_function = Opti {cache, multiplier: 0.5};

    let solver = ParticleSwarm::new((vec![-4.0, -4.0], vec![4.0, 4.0]), 40);

    let res = Executor::new(cost_function, solver)
        .configure(|state| state.max_iters(100))
        .run().unwrap();

    // Print Result
    println!("{res}");

}