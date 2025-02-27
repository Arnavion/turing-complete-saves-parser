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

parse_int! { i8, u8, i16, u16, i32, u32, i64, u64 }

impl<'a, A, B> Parse<'a> for (A, B) where A: Parse<'a>, B: Parse<'a> {
	fn parse(input: &mut &'a [u8]) -> Self {
		let a = <_>::parse(input);
		let b = <_>::parse(input);
		(a, b)
	}
}

#[derive(Clone, Copy)]
pub struct Slice<'a, N, T> {
	inner: &'a [u8],
	len: std::marker::PhantomData<N>,
	element: std::marker::PhantomData<T>,
}

impl<'a, N, T> Slice<'a, N, T> where T: Parse<'a> {
	pub fn until_end(inner: &'a [u8]) -> Self {
		Self {
			inner,
			len: Default::default(),
			element: Default::default(),
		}
	}

	pub fn iter(&self) -> SliceIter<'a, T> {
		SliceIter {
			inner: self.inner,
			element: self.element,
		}
	}
}

impl<'a, N, T> std::fmt::Debug for Slice<'a, N, T> where T: std::fmt::Debug + Parse<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut f = f.debug_list();
		for element in self {
			f.entry(&element);
		}
		f.finish()
	}
}

impl<'a, N, T> IntoIterator for &Slice<'a, N, T> where T: Parse<'a> {
	type Item = T;
	type IntoIter = SliceIter<'a, T>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, N, T> Parse<'a> for Slice<'a, N, T>
where
	N: Parse<'a>,
	usize: TryFrom<N>,
	<usize as TryFrom<N>>::Error: std::fmt::Debug,
	T: Parse<'a>
{
	fn parse(input: &mut &'a [u8]) -> Self {
		let len = usize::try_from(N::parse(input)).unwrap();
		let original_input = *input;

		for _ in 0..len {
			drop(T::parse(input));
		}

		let consumed_input = &original_input[..(original_input.len() - input.len())];

		Self {
			inner: consumed_input,
			len: Default::default(),
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
		let len = usize::from(u16::parse(input));
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
	};
}
