pub trait Parse<'a>: Sized {
	fn parse(input: &mut &'a [u8]) -> Self;
}

pub trait Encode {
	fn encode(&self, out: &mut Vec<u8>);
}

pub trait EncodeInt {
	fn encode_le_bytes(self, out: &mut [u8]);
}

impl Parse<'_> for bool {
	fn parse(input: &mut &[u8]) -> Self {
		let (result, rest) = input.split_first().unwrap();
		let result = *result != 0;
		*input = rest;
		result
	}
}

impl Encode for bool {
	fn encode(&self, out: &mut Vec<u8>) {
		u8::from(*self).encode(out);
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

			impl Encode for $ty {
				fn encode(&self, out: &mut Vec<u8>) {
					out.extend(self.to_le_bytes());
				}
			}

			impl EncodeInt for $ty {
				fn encode_le_bytes(self, out: &mut [u8]) {
					out.copy_from_slice(&self.to_le_bytes());
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

impl<A, B> Encode for (A, B) where A: Encode, B: Encode {
	fn encode(&self, out: &mut Vec<u8>) {
		self.0.encode(out);
		self.1.encode(out);
	}
}

impl<T> Encode for Option<T> where T: Encode {
	fn encode(&self, out: &mut Vec<u8>) {
		if let Some(this) = self {
			this.encode(out);
		}
	}
}

#[derive(Clone)]
pub struct Slice<'a, N, T> {
	inner: Either<&'a [u8], Vec<T>>,
	len: std::marker::PhantomData<N>,
	element: std::marker::PhantomData<T>,
}

impl<'a, N, T> Slice<'a, N, T> where T: Parse<'a> {
	pub fn parse_until_end(inner: &mut &'a [u8]) -> Self {
		Self {
			inner: Either::Left(std::mem::take(inner)),
			len: Default::default(),
			element: Default::default(),
		}
	}

	pub fn iter(&self) -> SliceIter<'a, '_, T> {
		SliceIter {
			inner: match &self.inner {
				Either::Left(inner) => Either::Left(inner),
				Either::Right(inner) => Either::Right(inner.iter()),
			},
			element: self.element,
		}
	}
}

impl<N, T> Slice<'_, N, T> {
	pub fn iter_mut(&mut self) -> SliceIterMut<'_, T> {
		let Either::Right(inner) = &mut self.inner else { panic!(); };
		SliceIterMut {
			inner: inner.iter_mut(),
		}
	}
}

impl<'a, N, T> Slice<'a, N, T>
where
	N: Parse<'a>,
	N: TryInto<usize>,
	<N as TryInto<usize>>::Error: std::fmt::Debug,
	T: Parse<'a>
{
	pub fn parse_with_length_prefix(input: &mut &'a [u8]) -> Self {
		let len: usize = N::parse(input).try_into().unwrap();
		let original_input = *input;

		for _ in 0..len {
			drop(T::parse(input));
		}

		let consumed_input = &original_input[..(original_input.len() - input.len())];

		Self {
			inner: Either::Left(consumed_input),
			len: Default::default(),
			element: Default::default(),
		}
	}
}

impl<'a, N, T> Slice<'a, N, T>
where
	N: EncodeInt,
	usize: TryInto<N>,
	<usize as TryInto<N>>::Error: std::fmt::Debug,
	T: Parse<'a> + Encode
{
	pub fn encode_with_length_prefix(&self, out: &mut Vec<u8>) {
		let index_pos = out.len();
		out.extend(std::iter::repeat(0).take(std::mem::size_of::<N>()));
		let element_start_pos = out.len();

		let mut len = 0_usize;
		for element in self {
			match element {
				Either::Left(element) => element.encode(out),
				Either::Right(element) => element.encode(out),
			}
			len += 1;
		}

		let len: N = len.try_into().unwrap();
		len.encode_le_bytes(&mut out[index_pos..element_start_pos]);
	}

	pub fn encode_without_length_prefix(&self, out: &mut Vec<u8>) {
		for element in self {
			match element {
				Either::Left(element) => element.encode(out),
				Either::Right(element) => element.encode(out),
			}
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

impl<N, T> From<Vec<T>> for Slice<'_, N, T> {
	fn from(inner: Vec<T>) -> Self {
		Self {
			inner: Either::Right(inner),
			len: Default::default(),
			element: Default::default(),
		}
	}
}

impl<'a, 'this, N, T> IntoIterator for &'this Slice<'a, N, T> where T: Parse<'a> {
	type Item = Either<T, &'this T>;
	type IntoIter = SliceIter<'a, 'this, T>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'this, N, T> IntoIterator for &'this mut Slice<'_, N, T> {
	type Item = &'this mut T;
	type IntoIter = SliceIterMut<'this, T>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

pub struct SliceIter<'a, 'this, T> {
	inner: Either<&'a [u8], std::slice::Iter<'this, T>>,
	element: std::marker::PhantomData<T>,
}

impl<'a, 'this, T> Iterator for SliceIter<'a, 'this, T> where T: Parse<'a> {
	type Item = Either<T, &'this T>;

	fn next(&mut self) -> Option<Self::Item> {
		match &mut self.inner {
			Either::Left(inner) => if inner.is_empty() {
				None
			}
			else {
				Some(Either::Left(T::parse(inner)))
			},

			Either::Right(inner) => inner.next().map(Either::Right),
		}
	}
}

pub struct SliceIterMut<'a, T> {
	inner: std::slice::IterMut<'a, T>,
}

impl<'a, T> Iterator for SliceIterMut<'a, T> {
	type Item = &'a mut T;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next()
	}
}

impl<'a> Parse<'a> for &'a str {
	fn parse(input: &mut &'a [u8]) -> Self {
		let len = usize::from(u16::parse(input));
		let (result, rest) = input.split_at(len);
		let result = str::from_utf8(result).unwrap();
		*input = rest;
		result
	}
}

impl Encode for str {
	fn encode(&self, out: &mut Vec<u8>) {
		let len: u16 = self.len().try_into().unwrap();
		len.encode(out);
		out.extend(self.bytes());
	}
}

impl Encode for &str {
	fn encode(&self, out: &mut Vec<u8>) {
		str::encode(*self, out);
	}
}

#[derive(Clone, Debug)]
pub enum Either<L, R> {
	Left(L),
	Right(R),
}

impl<L, R> Either<L, R> {
	pub fn as_ref(&self) -> Either<&L, &R> {
		match self {
			Self::Left(inner) => Either::Left(inner),
			Self::Right(inner) => Either::Right(inner),
		}
	}

	pub fn parse_left<'a>(input: &mut &'a [u8]) -> Self where L: Parse<'a> {
		Self::Left(<_>::parse(input))
	}

	pub fn parse_right<'a>(input: &mut &'a [u8]) -> Self where L: Parse<'a> {
		Self::Left(<_>::parse(input))
	}
}

impl<T> Either<T, T> {
	pub fn collapse(self) -> T {
		match self {
			Self::Left(inner) |
			Self::Right(inner) => inner,
		}
	}
}

impl<T> Either<T, &T> {
	pub fn as_inner_ref(&self) -> &T {
		match self {
			Self::Left(inner) => inner,
			Self::Right(inner) => inner,
		}
	}
}

impl<L, R> Iterator for Either<L, R> where L: Iterator, R: Iterator {
	type Item = Either<L::Item, R::Item>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Left(inner) => inner.next().map(Either::Left),
			Self::Right(inner) => inner.next().map(Either::Right),
		}
	}
}

impl<L, R> Encode for Either<L, R> where L: Encode, R: Encode {
	fn encode(&self, out: &mut Vec<u8>) {
		match self {
			Self::Left(inner) => inner.encode(out),
			Self::Right(inner) => inner.encode(out),
		}
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
					$($field_value => Self::$field_name,)*
					_ => unreachable!("{raw:?}"),
				}
			}
		}

		impl From<$enum_name> for $repr_ty {
			fn from(value: $enum_name) -> Self {
				match value {
					$(<$enum_name>::$field_name => $field_value,)*
				}
			}
		}

		impl<'a> $crate::Parse<'a> for $enum_name {
			fn parse(input: &mut &'a [u8]) -> Self {
				let raw = <$repr_ty>::parse(input);
				raw.into()
			}
		}

		impl $crate::Encode for $enum_name {
			fn encode(&self, out: &mut Vec<u8>) {
				let raw = <$repr_ty>::from(*self);
				raw.encode(out);
			}
		}
	};
}
