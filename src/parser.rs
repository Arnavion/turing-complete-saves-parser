#[derive(Debug)]
pub enum Either<L, R> {
	Left(L),
	Right(R),
}

pub trait Parse<'a>: Sized {
	fn parse(input: &mut &'a [u8]) -> Self;
}

impl Parse<'_> for bool {
	fn parse(input: &mut &[u8]) -> Self {
		let (result, rest) = input.split_first().unwrap();
		let result = *result != 0;
		*input = rest;
		result
	}
}

macro_rules! parse_int {
	($($ty:ty),*) => {
		$(
			impl Parse<'_> for $ty {
				fn parse(input: &mut &[u8]) -> Self {
					let (result, rest) = input.split_at(std::mem::size_of::<$ty>());
					let result = <$ty>::from_le_bytes(result.try_into().unwrap());
					*input = rest;
					result
				}
			}
		)*
	};
}

parse_int! { i64, u64, u32, u16, u8, i8 }

pub struct Slice<'a, T> {
	inner: &'a [u8],
	element: std::marker::PhantomData<T>,
}

impl<'a, T> Slice<'a, T> {
	#[allow(clippy::iter_not_returning_iterator)]
	pub fn iter(&self) -> SliceIter<'a, T> {
		SliceIter {
			inner: self.inner,
			element: self.element,
		}
	}
}

impl<'a, T> std::fmt::Debug for Slice<'a, T> where T: std::fmt::Debug + Parse<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut f = f.debug_list();
		for element in self.iter() {
			f.entry(&element);
		}
		f.finish()
	}
}

impl<'a, T> Parse<'a> for Slice<'a, T> where T: Parse<'a> {
	fn parse(input: &mut &'a [u8]) -> Self {
		let len = i64::parse(input).try_into().unwrap();
		let original_input = *input;

		for _ in 0..len {
			drop(T::parse(input));
		}

		let consumed_input = &original_input[..(original_input.len() - input.len())];

		Slice {
			inner: consumed_input,
			element: Default::default(),
		}
	}
}

pub struct SliceIter<'a, T> {
	inner: &'a [u8],
	element: std::marker::PhantomData<T>,
}

impl<'a, T> Iterator for SliceIter<'a, T> where T: Parse<'a> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		if self.inner.is_empty() {
			None
		}
		else {
			Some(T::parse(&mut self.inner))
		}
	}
}

impl<'a> Parse<'a> for &'a str {
	fn parse(input: &mut &'a [u8]) -> Self {
		let len = i64::parse(input).try_into().unwrap();
		let (result, rest) = input.split_at(len);
		let result = std::str::from_utf8(result).unwrap();
		*input = rest;
		result
	}
}

macro_rules! enum_impl_from {
	(
		$(#[$($meta:meta)*])*
		pub enum $enum_name:ident : $repr_ty:ty {
			$($field_name:ident = $field_value:literal,)*
		}
	) => {
		$(#[$($meta)*])*
		pub enum $enum_name {
			$($field_name = $field_value,)*
		}

		impl From<$repr_ty> for $enum_name {
			fn from(raw: $repr_ty) -> Self {
				match raw {
					$($field_value => $enum_name :: $field_name,)*
					_ => unreachable!("{raw:?}"),
				}
			}
		}

		impl From<&'_ str> for $enum_name {
			fn from(s: &str) -> Self {
				match s {
					$(stringify!($field_name) => $enum_name :: $field_name,)*
					_ => unreachable!("{s:?}"),
				}
			}
		}
	};
}
