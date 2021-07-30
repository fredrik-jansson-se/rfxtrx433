#![allow(missing_docs)]
use bitflags::bitflags;

bitflags! {
    /// Protocols
    pub struct Protocols1:u8 {
        /// AE Blyss
        const AE = 1<<0;
        /// Rubicson, Lacrosse, Banggood
        const RUBICSON = 1<<1;
        /// Fineoffset, Viking
        const FINEOFFSET = 1<<2;
        /// PT2262 and compatible
        const LIGHTING4 = 1<<3;
        /// RSL, Revolt
        const RSL = 1<<4;
        /// ByronSX, Selectplus
        const SX = 1<<5;
        /// Imagintronix, Opus
        const IMAGINTRONIX = 1<<6;
        /// Undecoded messages
        const UNDECODED = 1<<7;
    }
}

bitflags! {
    /// Protocols
    pub struct Protocols2:u8 {
        /// Mertik maxitrol
        const	MERTIK = 1<<0;
        /// AD LightwaveRF
        const	LWRF = 1<<1;
        /// Hideki
        const	HIDEKI = 1<<2;
        /// LaCrosse
        const	LACROSSE = 1<<3;
        /// Legrand CAD
        const	LEGRAND = 1<<4;
        /// Reserved for future use
        const	MSG4_RESERVED_55 = 1<<5;
        /// Rollertrol, Hasta new
        const	BLINDST0 = 1<<6;
        /// BlindsT1-4
        const	BLINDST1 = 1<<7;
    }
}

bitflags! {
    /// Protocols
    pub struct Protocols3:u8 {
        /// X10
        const X10 = 1<<0;
        /// ARC
        const ARC = 1<<1;
        /// AC
        const AC  = 1<<2;
        /// HomeEasy EU
        const HEEU = 1<<3;
        /// Meiantech,Atlantic
        const MEIANTECH = 1<<4;
        /// Oregon Scientific
        const OREGON = 1<<5;
        /// ATI remotes
        const ATI    = 1<<6;
        /// Visonic PowerCode
        const VISONIC = 1<<7;
    }
}

bitflags! {
    /// Protocols
    pub struct Protocols4:u8 {
        /// Keeloq

        const KEELOQ= 1 << 0;
        /// HomeConfort
        const HC= 1 << 1;
        /// Reserved for future use
        const MSG6_RESERVED_2 = 1<<2;
        /// Reserved for future use
        const MSG6_RESERVED_3 = 1<<3;
        /// Reserved for future use
        const MSG6_RESERVED_4 = 1<<4;
        /// Reserved for future use
        const MSG6_RESERVED_5 = 1<<5;
        /// MCZ
        const MCZ = 1<<6;
        /// Funkbus
        const FUNKBUS = 1<<7;
    }
}
