use common::units::Milimeters;
use nalgebra::Vector2;

/// One Creality printer that reads CXDLP files.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CrealityPrinter {
    pub model: &'static str,
    pub resolution: (u32, u32),
    pub build_height_mm: f32,
}

const PRINTERS: [CrealityPrinter; 7] = [
    CrealityPrinter {
        model: "CL-79",
        resolution: (4320, 2560),
        build_height_mm: 160.0,
    },
    CrealityPrinter {
        model: "CL-60",
        resolution: (1620, 2560),
        build_height_mm: 160.0,
    },
    CrealityPrinter {
        model: "CL-70",
        resolution: (2560, 2400),
        build_height_mm: 160.0,
    },
    CrealityPrinter {
        model: "CL-89",
        resolution: (3840, 2400),
        build_height_mm: 200.0,
    },
    CrealityPrinter {
        model: "CL-92",
        resolution: (5760, 3600),
        build_height_mm: 210.0,
    },
    CrealityPrinter {
        model: "CL-103",
        resolution: (7680, 4320),
        build_height_mm: 230.0,
    },
    CrealityPrinter {
        model: "CL-133",
        resolution: (3840, 2160),
        build_height_mm: 300.0,
    },
];

impl CrealityPrinter {
    /// Return the printer that has the given screen resolution.
    pub fn from_resolution(resolution: Vector2<u32>) -> Option<Self> {
        PRINTERS
            .into_iter()
            .find(|printer| printer.resolution == (resolution.x, resolution.y))
    }

    /// Return the printer with the given Creality model code, such as `CL-79`.
    pub fn from_model(model: &str) -> Option<Self> {
        PRINTERS.into_iter().find(|printer| printer.model == model)
    }

    /// Return the build height of the printer.
    pub fn build_height(&self) -> Milimeters {
        Milimeters::new(self.build_height_mm)
    }
}
