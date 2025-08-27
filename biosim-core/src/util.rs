pub struct DOption<T>(pub bool, pub T) where T: Default;


impl<T> DOption<T>
where T: Default
{
    pub fn some(item: T) -> DOption<T> {
        DOption(true, item)
    }

    pub fn none() -> DOption<T> {
        DOption(false, Default::default())
    }

    pub fn unwrap_or_default(self, default: T) -> T {
        match self {
            DOption(true, item) => item,
            DOption(false, _) => default
        }
    }
}

