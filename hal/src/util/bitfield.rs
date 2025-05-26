#[macro_export]
macro_rules! bitfield_access {
    ($reg_type:ty, $field:ident, $range:expr, RO) => {
        pub fn $field(&self) -> $reg_type {
            let mask = ((1 << ($range.end - $range.start + 1)) - 1) << $range.start;
            ((self.value & mask) >> $range.start) as $reg_type
        }
    };
    ($reg_type:ty, $field:ident, $range:expr, RW) => {
        pub fn $field(&self) -> $reg_type {
            let mask = ((1 << ($range.end - $range.start + 1)) - 1) << $range.start;
            ((self.value & mask) >> $range.start) as $reg_type
        }
        paste::paste!{
            pub fn [<set_ $field>](&mut self, val: $reg_type) {
                let mask = ((1 << ($range.end - $range.start + 1)) - 1) << ($range.start);
                self.value = (self.value & !mask) | ((val as $reg_type) << ($range.start));
            }
        }
    };

    ($reg_type:ty, $field:ident, $range:expr, WO) => {
        paste::paste!{
            pub fn [<set_ $field>](&mut self, val: $reg_type) {
                let mask = ((1 << ($range.end - $range.start + 1)) - 1) << ($range.start);
                self.value = (self.value & !mask) | ((val as $reg_type) << ($range.start));
            }
        }
    };
}

// 位域访问宏
#[macro_export]
macro_rules! bitfield {
    ($name:ident<$reg_type:ty> {
        $($field:ident($range:expr) [$access:ident]),+
    }) => {
        #[repr(transparent)]
        #[derive(Copy, Clone)]
        pub struct $name {
            value: $reg_type,
        }
        
        impl $name {
            $(
                crate::bitfield_access!($reg_type, $field, $range, $access);
            )+
            
            pub fn from_bits(bits: $reg_type) -> Self {
                Self { value: bits }
            }
            
            pub fn bits(&self) -> $reg_type {
                self.value
            }
        }
    };
}

