use cgmath::*;

pub trait Matrix4Ext<S> {
    fn truncate(self) -> Matrix3<S>;
}

impl<S: cgmath::BaseNum> Matrix4Ext<S> for Matrix4<S> {
    fn truncate(self) -> Matrix3<S> {
        Matrix3::from_cols(
            self[0].truncate(),
            self[1].truncate(),
            self[2].truncate(),
        )
    }
}
