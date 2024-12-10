use std::io;

use thiserror::Error;

use crate::four_cc::FourCC;

/// Represents an error when initializing a synthesizer.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SynthesizerError {
    #[error("the sample rate must be between 16000 and 192000, but was {0}")]
    SampleRateOutOfRange(i32),
    #[error("the block size must be between 8 and 1024, but was {0}")]
    BlockSizeOutOfRange(usize),
    #[error("the maximum number of polyphony must be between 8 and 256, but was {0}")]
    MaximumPolyphonyOutOfRange(usize),
}

/// Represents an error when loading a SoundFont.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ParseError {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error("the RIFF chunk was not found")]
    RiffChunkNotFound,
    #[error("the type of the RIFF chunk must be '{expected}', but was '{actual}'")]
    InvalidRiffChunkType { expected: FourCC, actual: FourCC },
    #[error("the LIST chunk was not found")]
    ListChunkNotFound,
    #[error("the type of the LIST chunk must be '{expected}', but was '{actual}'")]
    InvalidListChunkType { expected: FourCC, actual: FourCC },
    #[error("the INFO list contains an unknown ID '{0}'")]
    ListContainsUnknownId(FourCC),
    #[error("no valid sample data was found")]
    SampleDataNotFound,
    #[error("SoundFont3 is not yet supported")]
    UnsupportedSampleFormat,
    #[error("the '{0}' sub-chunk was not found")]
    SubChunkNotFound(FourCC),
    #[error("the preset list is invalid")]
    InvalidPresetList,
    #[error(
        "the preset with the ID '{preset_id}' contains an invalid instrument ID '{instrument_id}'"
    )]
    InvalidInstrumentId {
        preset_id: usize,
        instrument_id: usize,
    },
    #[error("the preset with the ID '{0}' has no zone")]
    InvalidPreset(usize),
    #[error("no valid preset was found")]
    PresetNotFound,
    #[error("the instrument list is invalid")]
    InvalidInstrumentList,
    #[error(
        "the instrument with the ID '{instrument_id}' contains an invalid sample ID '{sample_id}'"
    )]
    InvalidSampleId {
        instrument_id: usize,
        sample_id: usize,
    },
    #[error("the instrument with the ID '{0}' has no zone")]
    InvalidInstrument(usize),
    #[error("no valid instrument was found")]
    InstrumentNotFound,
    #[error("the sample header list is invalid")]
    InvalidSampleHeaderList,
    #[error("the zone list is invalid")]
    InvalidZoneList,
    #[error("no valid zone was found")]
    ZoneNotFound,
    #[error("the generator list is invalid")]
    InvalidGeneratorList,
    #[error("sanity check failed")]
    SanityCheckFailed,
}

/// Represents an error when loading a MIDI file.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum MidiFileError {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error("the chunk type must be '{expected}', but was '{actual}'")]
    InvalidChunkType { expected: FourCC, actual: FourCC },
    #[error("the '{0}' chunk has invalid data")]
    InvalidChunkData(FourCC),
    #[error("the format {0} is not supported")]
    UnsupportedFormat(i16),
    #[error("failed to read the tempo value")]
    InvalidTempoValue,
}
