#![allow(dead_code)]

use std::io::Read;
use std::sync::Arc;

use crate::binary_reader::BinaryReader;
use crate::error::SoundFontError;
use crate::four_cc::FourCC;
use crate::instrument::Instrument;
use crate::preset::Preset;
use crate::sample_header::SampleHeader;
use crate::soundfont_info::SoundFontInfo;
use crate::soundfont_parameters::SoundFontParameters;
use crate::soundfont_sampledata::SoundFontSampleData;

/// Reperesents a SoundFont.
#[non_exhaustive]
pub struct SoundFont {
    pub(crate) info: SoundFontInfo,
    pub(crate) bits_per_sample: i32,
    pub(crate) wave_data: Arc<Vec<i16>>,
    pub(crate) sample_headers: Vec<SampleHeader>,
    pub(crate) presets: Vec<Preset>,
    pub(crate) instruments: Vec<Instrument>,
}

impl SoundFont {
    /// Loads a SoundFont from the stream.
    ///
    /// # Arguments
    ///
    /// * `reader` - The data stream used to load the SoundFont.
    pub fn new<R: Read>(reader: &mut R) -> Result<Self, SoundFontError> {
        let chunk_id = BinaryReader::read_four_cc(reader)?;
        if chunk_id != b"RIFF" {
            return Err(SoundFontError::RiffChunkNotFound);
        }

        let _size = BinaryReader::read_i32(reader);

        let form_type = BinaryReader::read_four_cc(reader)?;
        if form_type != b"sfbk" {
            return Err(SoundFontError::InvalidRiffChunkType {
                expected: FourCC::from_bytes(*b"sfbk"),
                actual: form_type,
            });
        }

        let info = SoundFontInfo::new(reader)?;
        let sample_data = SoundFontSampleData::new(reader)?;
        let parameters = SoundFontParameters::new(reader)?;

        Ok(Self {
            info,
            bits_per_sample: 16,
            wave_data: Arc::new(sample_data.wave_data),
            sample_headers: parameters.sample_headers,
            presets: parameters.presets,
            instruments: parameters.instruments,
        })
    }

    /// Gets the information of the SoundFont.
    pub fn get_info(&self) -> &SoundFontInfo {
        &self.info
    }

    /// Gets the bits per sample of the sample data.
    pub fn get_bits_per_sample(&self) -> i32 {
        self.bits_per_sample
    }

    /// Gets the sample data.
    pub fn get_wave_data(&self) -> &[i16] {
        &self.wave_data[..]
    }

    /// Gets the samples of the SoundFont.
    pub fn get_sample_headers(&self) -> &[SampleHeader] {
        &self.sample_headers[..]
    }

    /// Gets the presets of the SoundFont.
    pub fn get_presets(&self) -> &[Preset] {
        &self.presets[..]
    }

    /// Gets the instruments of the SoundFont.
    pub fn get_instruments(&self) -> &[Instrument] {
        &self.instruments[..]
    }
}

/// Notes:
///
/// - This works, but I wonder if we couldn't just use the original structures + maybe
///   something like `#[cfg_attr(feature = "export", derive(Serialize, Deserialize))]`.
///     - Computing preset hash could still be done without the access to the private
///       fields, through getters.
///     - The only problem is figuring out how to de/serialize `gs` which has `[i16; 61]`
///       type.
/// - Honestly, a bigger issue is "distilling" soundfont to a single preset. Doing so is
///   quite complex, as we need to keep track of all the instruments and samples that are
///   used by the preset, then filter out and remap indexes to the remaining instruments
///   and regions.
///     - In the end, the distillation may not be that hard. It would be hard right now,
///       because of all the noise coming from the duplicated structures. Like, I think
///       this problem isn't something that a few hashmaps couldn't solve.
#[cfg(feature = "export")]
pub mod export {
    use std::sync::Arc;

    use serde::{Deserialize, Serialize};

    use crate::generator_type::GeneratorType;

    impl super::SoundFont {
        pub fn export_preset(&self, bank: i32, patch: i32) -> Option<ExportedPreset> {
            let preset = self
                .presets
                .iter()
                .find(|p| p.bank_number == bank && p.patch_number == patch)?;

            Some(ExportedPreset {
                info: SoundFontInfo {
                    version: SoundFontVersion {
                        major: self.info.version.major,
                        minor: self.info.version.minor,
                    },
                    target_sound_engine: self.info.target_sound_engine.clone(),
                    bank_name: self.info.bank_name.clone(),
                    rom_name: self.info.rom_name.clone(),
                    rom_version: SoundFontVersion {
                        major: self.info.rom_version.major,
                        minor: self.info.rom_version.minor,
                    },
                    creation_date: self.info.creation_date.clone(),
                    author: self.info.author.clone(),
                    target_product: self.info.target_product.clone(),
                    copyright: self.info.copyright.clone(),
                    comments: self.info.comments.clone(),
                    tools: self.info.tools.clone(),
                },
                bits_per_sample: self.bits_per_sample,
                wave_data: (*self.wave_data).clone(),
                preset: Preset {
                    name: preset.name.clone(),
                    patch_number: preset.patch_number,
                    bank_number: preset.bank_number,
                    library: preset.library,
                    genre: preset.genre,
                    morphology: preset.morphology,
                    regions: preset
                        .regions
                        .iter()
                        .map(|r| PresetRegion {
                            gs: r.gs.into_iter().collect(),
                            instrument: r.instrument,
                        })
                        .collect(),
                },
                instruments: self
                    .instruments
                    .iter()
                    .map(|i| Instrument {
                        name: i.name.clone(),
                        regions: i
                            .regions
                            .iter()
                            .map(|r| InstrumentRegion {
                                gs: r.gs.into_iter().collect(),
                                sample_start: r.sample_start,
                                sample_end: r.sample_end,
                                sample_start_loop: r.sample_start_loop,
                                sample_end_loop: r.sample_end_loop,
                                sample_sample_rate: r.sample_sample_rate,
                                sample_original_pitch: r.sample_original_pitch,
                                sample_pitch_correction: r.sample_pitch_correction,
                            })
                            .collect(),
                    })
                    .collect(),
                regions: preset
                    .regions
                    .iter()
                    .map(|r| PresetRegion {
                        gs: r.gs.into_iter().collect(),
                        instrument: r.instrument,
                    })
                    .collect(),
                sample_headers: self
                    .sample_headers
                    .iter()
                    .map(|s| SampleHeader {
                        name: s.name.clone(),
                        start: s.start,
                        end: s.end,
                        start_loop: s.start_loop,
                        end_loop: s.end_loop,
                        sample_rate: s.sample_rate,
                        original_pitch: s.original_pitch,
                        pitch_correction: s.pitch_correction,
                        link: s.link,
                        sample_type: s.sample_type,
                    })
                    .collect(),
            })
        }

        pub fn from_exported_preset(preset: ExportedPreset) -> super::SoundFont {
            crate::SoundFont {
                info: crate::SoundFontInfo {
                    version: crate::SoundFontVersion {
                        major: preset.info.version.major,
                        minor: preset.info.version.minor,
                    },
                    target_sound_engine: preset.info.target_sound_engine,
                    bank_name: preset.info.bank_name,
                    rom_name: preset.info.rom_name,
                    rom_version: crate::SoundFontVersion {
                        major: preset.info.rom_version.major,
                        minor: preset.info.rom_version.minor,
                    },
                    creation_date: preset.info.creation_date,
                    author: preset.info.author,
                    target_product: preset.info.target_product,
                    copyright: preset.info.copyright,
                    comments: preset.info.comments,
                    tools: preset.info.tools,
                },
                bits_per_sample: preset.bits_per_sample,
                wave_data: Arc::new(preset.wave_data),
                presets: vec![crate::Preset {
                    name: preset.preset.name,
                    patch_number: preset.preset.patch_number,
                    bank_number: preset.preset.bank_number,
                    library: preset.preset.library,
                    genre: preset.preset.genre,
                    morphology: preset.preset.morphology,
                    regions: preset
                        .regions
                        .iter()
                        .map(|r| crate::PresetRegion {
                            gs: {
                                let mut gs = [0; GeneratorType::COUNT];
                                for (i, v) in r.gs.iter().enumerate() {
                                    gs[i] = *v;
                                }
                                gs
                            },
                            instrument: r.instrument,
                        })
                        .collect(),
                }],
                instruments: preset
                    .instruments
                    .iter()
                    .map(|i| crate::Instrument {
                        name: i.name.clone(),
                        regions: i
                            .regions
                            .iter()
                            .map(|r| crate::InstrumentRegion {
                                gs: {
                                    let mut gs = [0; GeneratorType::COUNT];
                                    for (i, v) in r.gs.iter().enumerate() {
                                        gs[i] = *v;
                                    }
                                    gs
                                },
                                sample_start: r.sample_start,
                                sample_end: r.sample_end,
                                sample_start_loop: r.sample_start_loop,
                                sample_end_loop: r.sample_end_loop,
                                sample_sample_rate: r.sample_sample_rate,
                                sample_original_pitch: r.sample_original_pitch,
                                sample_pitch_correction: r.sample_pitch_correction,
                            })
                            .collect(),
                    })
                    .collect(),
                sample_headers: preset
                    .sample_headers
                    .iter()
                    .map(|s| crate::SampleHeader {
                        name: s.name.clone(),
                        start: s.start,
                        end: s.end,
                        start_loop: s.start_loop,
                        end_loop: s.end_loop,
                        sample_rate: s.sample_rate,
                        original_pitch: s.original_pitch,
                        pitch_correction: s.pitch_correction,
                        link: s.link,
                        sample_type: s.sample_type,
                    })
                    .collect(),
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ExportedPreset {
        pub info: SoundFontInfo,
        pub bits_per_sample: i32,
        pub wave_data: Vec<i16>,
        pub preset: Preset,
        pub sample_headers: Vec<SampleHeader>,
        pub instruments: Vec<Instrument>,
        pub regions: Vec<PresetRegion>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SoundFontInfo {
        pub version: SoundFontVersion,
        pub target_sound_engine: String,
        pub bank_name: String,
        pub rom_name: String,
        pub rom_version: SoundFontVersion,
        pub creation_date: String,
        pub author: String,
        pub target_product: String,
        pub copyright: String,
        pub comments: String,
        pub tools: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SoundFontVersion {
        pub major: i16,
        pub minor: i16,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SampleHeader {
        pub name: String,
        pub start: i32,
        pub end: i32,
        pub start_loop: i32,
        pub end_loop: i32,
        pub sample_rate: i32,
        pub original_pitch: u8,
        pub pitch_correction: i8,
        pub link: u16,
        pub sample_type: u16,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Preset {
        pub name: String,
        pub patch_number: i32,
        pub bank_number: i32,
        pub library: i32,
        pub genre: i32,
        pub morphology: i32,
        pub regions: Vec<PresetRegion>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PresetRegion {
        pub gs: Vec<i16>,
        pub instrument: usize,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Instrument {
        pub name: String,
        pub regions: Vec<InstrumentRegion>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct InstrumentRegion {
        pub gs: Vec<i16>,
        pub sample_start: i32,
        pub sample_end: i32,
        pub sample_start_loop: i32,
        pub sample_end_loop: i32,
        pub sample_sample_rate: i32,
        pub sample_original_pitch: i32,
        pub sample_pitch_correction: i32,
    }

    #[cfg(test)]
    mod test {
        use std::sync::Arc;

        use crate::{SoundFont, Synthesizer, SynthesizerSettings};

        // Run test that compares rendered audio with expected audio for normal soundfont import.
        #[test]
        fn test_soundfont_import() {
            let mut file = std::fs::File::open(
                "/Users/iglak/Documents/Repos/sflap_test_soundfonts/EarthBound.sf2",
            )
            .unwrap();
            let sf = Arc::new(SoundFont::new(&mut file).unwrap());

            let presets = sf
                .get_presets()
                .iter()
                .map(|p| (p.bank_number, p.patch_number));

            for (bank, patch) in presets {
                test_patch(&sf, bank, patch);
            }
        }

        fn test_patch(sf: &Arc<SoundFont>, bank: i32, patch: i32) {
            let mut synth = Synthesizer::new(&sf, &SynthesizerSettings::new(44100)).unwrap();
            synth.set_bank(bank);
            synth.set_patch(patch);
            synth.note_on(0, 60, 127);
            let mut expected = ([0.; 1024], [0.; 1024]);
            synth.render(&mut expected.0, &mut expected.1);

            let exported = sf.export_preset(bank, patch).unwrap();

            dbg!(&exported);

            // Use this to check general memory usage.
            //
            // ciborium::into_writer(&exported,
            // std::fs::File::create("Touhou.sf2.ciborium").unwrap()).unwrap();

            let sf = Arc::new(SoundFont::from_exported_preset(exported));
            let mut synth = Synthesizer::new(&sf, &SynthesizerSettings::new(44100)).unwrap();
            synth.set_bank(bank);
            synth.set_patch(patch);
            synth.note_on(0, 60, 127);
            let mut actual = ([0.; 1024], [0.; 1024]);
            synth.render(&mut actual.0, &mut actual.1);

            assert_eq!(expected.0, actual.0, "Bank: {}, Patch: {}", bank, patch);
            assert_eq!(expected.1, actual.1, "Bank: {}, Patch: {}", bank, patch);
        }
    }
}
