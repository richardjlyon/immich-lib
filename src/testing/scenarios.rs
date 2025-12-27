//! Test scenario types for duplicate group categorization.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Test scenarios for categorizing duplicate groups.
///
/// Each scenario represents a specific test case that needs coverage
/// in the integration test suite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TestScenario {
    // Winner selection scenarios (W)
    /// Clear dimension winner (different width x height)
    W1ClearDimensionWinner,
    /// Same dimensions, different file size
    W2SameDimensionsDifferentSize,
    /// Same dimensions, same file size
    W3SameDimensionsSameSize,
    /// Some assets missing dimensions
    W4SomeMissingDimensions,
    /// Only one asset has dimensions
    W5OnlyOneHasDimensions,
    /// All assets missing dimensions
    W6AllMissingDimensions,
    /// 3+ assets in group
    W7ThreePlusDuplicates,
    /// Same pixel count, different aspect ratio
    W8SamePixelsDifferentAspect,

    // Consolidation scenarios (C)
    /// Winner lacks GPS, loser has GPS
    C1WinnerLacksGpsLoserHas,
    /// Winner lacks datetime, loser has datetime
    C2WinnerLacksDatetimeLoserHas,
    /// Winner lacks description, loser has description
    C3WinnerLacksDescriptionLoserHas,
    /// Winner lacks all three, loser has all
    C4WinnerLacksAllLoserHasAll,
    /// Both have GPS (no consolidation needed)
    C5BothHaveGps,
    /// Multiple losers contribute different fields
    C6MultipleLosersContribute,
    /// No loser has what winner lacks
    C7NoLoserHasNeeded,
    /// Winner already has everything
    C8WinnerHasEverything,

    // Conflict scenarios (F)
    /// GPS conflict (different locations)
    F1GpsConflict,
    /// GPS within threshold (should NOT conflict)
    F2GpsWithinThreshold,
    /// Timezone conflict
    F3TimezoneConflict,
    /// Camera info conflict
    F4CameraConflict,
    /// Capture time conflict
    F5CaptureTimeConflict,
    /// Multiple conflicts
    F6MultipleConflicts,
    /// No conflicts
    F7NoConflicts,

    // Edge case scenarios (X)
    /// Single asset "group"
    X1SingleAssetGroup,
    /// Large group (10+ duplicates)
    X2LargeGroup,
    /// Large file (>50MB)
    X3LargeFile,
    /// Special characters in filename
    X4SpecialCharsFilename,
    /// Video duplicates
    X5Video,
    /// PNG files (limited EXIF)
    X7Png,
    /// Unicode in description
    X9UnicodeDescription,
    /// Very old date (<1990)
    X10VeryOldDate,
    /// Future date
    X11FutureDate,
}

impl TestScenario {
    /// Returns all test scenarios.
    pub fn all() -> Vec<TestScenario> {
        vec![
            // Winner selection
            Self::W1ClearDimensionWinner,
            Self::W2SameDimensionsDifferentSize,
            Self::W3SameDimensionsSameSize,
            Self::W4SomeMissingDimensions,
            Self::W5OnlyOneHasDimensions,
            Self::W6AllMissingDimensions,
            Self::W7ThreePlusDuplicates,
            Self::W8SamePixelsDifferentAspect,
            // Consolidation
            Self::C1WinnerLacksGpsLoserHas,
            Self::C2WinnerLacksDatetimeLoserHas,
            Self::C3WinnerLacksDescriptionLoserHas,
            Self::C4WinnerLacksAllLoserHasAll,
            Self::C5BothHaveGps,
            Self::C6MultipleLosersContribute,
            Self::C7NoLoserHasNeeded,
            Self::C8WinnerHasEverything,
            // Conflicts
            Self::F1GpsConflict,
            Self::F2GpsWithinThreshold,
            Self::F3TimezoneConflict,
            Self::F4CameraConflict,
            Self::F5CaptureTimeConflict,
            Self::F6MultipleConflicts,
            Self::F7NoConflicts,
            // Edge cases
            Self::X1SingleAssetGroup,
            Self::X2LargeGroup,
            Self::X3LargeFile,
            Self::X4SpecialCharsFilename,
            Self::X5Video,
            Self::X7Png,
            Self::X9UnicodeDescription,
            Self::X10VeryOldDate,
            Self::X11FutureDate,
        ]
    }

    /// Returns the short code (e.g., "w1", "c2", "f3", "x5").
    pub fn code(&self) -> &'static str {
        match self {
            Self::W1ClearDimensionWinner => "w1",
            Self::W2SameDimensionsDifferentSize => "w2",
            Self::W3SameDimensionsSameSize => "w3",
            Self::W4SomeMissingDimensions => "w4",
            Self::W5OnlyOneHasDimensions => "w5",
            Self::W6AllMissingDimensions => "w6",
            Self::W7ThreePlusDuplicates => "w7",
            Self::W8SamePixelsDifferentAspect => "w8",
            Self::C1WinnerLacksGpsLoserHas => "c1",
            Self::C2WinnerLacksDatetimeLoserHas => "c2",
            Self::C3WinnerLacksDescriptionLoserHas => "c3",
            Self::C4WinnerLacksAllLoserHasAll => "c4",
            Self::C5BothHaveGps => "c5",
            Self::C6MultipleLosersContribute => "c6",
            Self::C7NoLoserHasNeeded => "c7",
            Self::C8WinnerHasEverything => "c8",
            Self::F1GpsConflict => "f1",
            Self::F2GpsWithinThreshold => "f2",
            Self::F3TimezoneConflict => "f3",
            Self::F4CameraConflict => "f4",
            Self::F5CaptureTimeConflict => "f5",
            Self::F6MultipleConflicts => "f6",
            Self::F7NoConflicts => "f7",
            Self::X1SingleAssetGroup => "x1",
            Self::X2LargeGroup => "x2",
            Self::X3LargeFile => "x3",
            Self::X4SpecialCharsFilename => "x4",
            Self::X5Video => "x5",
            Self::X7Png => "x7",
            Self::X9UnicodeDescription => "x9",
            Self::X10VeryOldDate => "x10",
            Self::X11FutureDate => "x11",
        }
    }

    /// Returns the category prefix (W, C, F, or X).
    pub fn category(&self) -> &'static str {
        match self {
            Self::W1ClearDimensionWinner
            | Self::W2SameDimensionsDifferentSize
            | Self::W3SameDimensionsSameSize
            | Self::W4SomeMissingDimensions
            | Self::W5OnlyOneHasDimensions
            | Self::W6AllMissingDimensions
            | Self::W7ThreePlusDuplicates
            | Self::W8SamePixelsDifferentAspect => "Winner Selection",
            Self::C1WinnerLacksGpsLoserHas
            | Self::C2WinnerLacksDatetimeLoserHas
            | Self::C3WinnerLacksDescriptionLoserHas
            | Self::C4WinnerLacksAllLoserHasAll
            | Self::C5BothHaveGps
            | Self::C6MultipleLosersContribute
            | Self::C7NoLoserHasNeeded
            | Self::C8WinnerHasEverything => "Consolidation",
            Self::F1GpsConflict
            | Self::F2GpsWithinThreshold
            | Self::F3TimezoneConflict
            | Self::F4CameraConflict
            | Self::F5CaptureTimeConflict
            | Self::F6MultipleConflicts
            | Self::F7NoConflicts => "Conflicts",
            Self::X1SingleAssetGroup
            | Self::X2LargeGroup
            | Self::X3LargeFile
            | Self::X4SpecialCharsFilename
            | Self::X5Video
            | Self::X7Png
            | Self::X9UnicodeDescription
            | Self::X10VeryOldDate
            | Self::X11FutureDate => "Edge Cases",
        }
    }
}

impl fmt::Display for TestScenario {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::W1ClearDimensionWinner => "W1: Clear dimension winner",
            Self::W2SameDimensionsDifferentSize => "W2: Same dimensions, different size",
            Self::W3SameDimensionsSameSize => "W3: Same dimensions, same size",
            Self::W4SomeMissingDimensions => "W4: Some missing dimensions",
            Self::W5OnlyOneHasDimensions => "W5: Only one has dimensions",
            Self::W6AllMissingDimensions => "W6: All missing dimensions",
            Self::W7ThreePlusDuplicates => "W7: 3+ duplicates",
            Self::W8SamePixelsDifferentAspect => "W8: Same pixels, different aspect",
            Self::C1WinnerLacksGpsLoserHas => "C1: Winner lacks GPS, loser has",
            Self::C2WinnerLacksDatetimeLoserHas => "C2: Winner lacks datetime, loser has",
            Self::C3WinnerLacksDescriptionLoserHas => "C3: Winner lacks description, loser has",
            Self::C4WinnerLacksAllLoserHasAll => "C4: Winner lacks all, loser has all",
            Self::C5BothHaveGps => "C5: Both have GPS",
            Self::C6MultipleLosersContribute => "C6: Multiple losers contribute",
            Self::C7NoLoserHasNeeded => "C7: No loser has needed",
            Self::C8WinnerHasEverything => "C8: Winner has everything",
            Self::F1GpsConflict => "F1: GPS conflict",
            Self::F2GpsWithinThreshold => "F2: GPS within threshold",
            Self::F3TimezoneConflict => "F3: Timezone conflict",
            Self::F4CameraConflict => "F4: Camera conflict",
            Self::F5CaptureTimeConflict => "F5: Capture time conflict",
            Self::F6MultipleConflicts => "F6: Multiple conflicts",
            Self::F7NoConflicts => "F7: No conflicts",
            Self::X1SingleAssetGroup => "X1: Single asset group",
            Self::X2LargeGroup => "X2: Large group (10+)",
            Self::X3LargeFile => "X3: Large file (>50MB)",
            Self::X4SpecialCharsFilename => "X4: Special chars in filename",
            Self::X5Video => "X5: Video",
            Self::X7Png => "X7: PNG",
            Self::X9UnicodeDescription => "X9: Unicode description",
            Self::X10VeryOldDate => "X10: Very old date (<1990)",
            Self::X11FutureDate => "X11: Future date",
        };
        write!(f, "{}", name)
    }
}

/// A match between a test scenario and a duplicate group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioMatch {
    /// The matched scenario
    pub scenario: TestScenario,
    /// Duplicate group ID
    pub duplicate_id: String,
    /// Description of why this matched
    pub details: String,
}
