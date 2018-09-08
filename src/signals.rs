bitflags! {
    pub struct Signal: u32 {
        const READABLE = 1 << 0;
        const WRITABLE = 1 << 1;
        const PEER_CLOSED =     1 << 2;
        const PEER_SIGNALED =   1 << 3;
        const EVENT_SIGNALED = 1 << 4;
        // ...
        const HANDLE_CLOSED =   1 << 5;
        // user signals
        const USER_0 =          1 << 24;
        const USER_1 =          1 << 25;
        const USER_2 =          1 << 26;
        const USER_3 =          1 << 27;
        const USER_4 =          1 << 28;
        const USER_5 =          1 << 29;
        const USER_6 =          1 << 30;
        const USER_7 =          1 << 31;

        const USER_ALL =
              Self::USER_0.bits
            | Self::USER_1.bits
            | Self::USER_2.bits
            | Self::USER_3.bits
            | Self::USER_4.bits
            | Self::USER_5.bits
            | Self::USER_6.bits
            | Self::USER_7.bits;
    }
}
