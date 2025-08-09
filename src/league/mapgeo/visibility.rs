use binrw::BinRead;
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct EnvironmentVisibility: u8 {
        const NoLayer = 0;
        const Layer1 = 1 << 0;
        const Layer2 = 1 << 1;
        const Layer3 = 1 << 2;
        const Layer4 = 1 << 3;
        const Layer5 = 1 << 4;
        const Layer6 = 1 << 5;
        const Layer7 = 1 << 6;
        const Layer8 = 1 << 7;
        const AllLayers = 255;
    }
}

impl BinRead for EnvironmentVisibility {
    type Args<'a> = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let flags = u8::read_options(reader, endian, ())?;
        EnvironmentVisibility::from_bits(flags).ok_or_else(|| {
            binrw::Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid environment visibility flags: {}", flags),
            ))
        })
    }
}
