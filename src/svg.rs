use crate::storage::Stream;

multiversx_sc::imports!();

const LOOKUP_TABLE: [u8; 64] = [
    65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88,
    89, 90, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114,
    115, 116, 117, 118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 43, 47,
];
const PADDING: u8 = 61;

#[multiversx_sc::module]
pub trait SvgModule:
    crate::storage::StorageModule
    + crate::events::EventsModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    fn base64(&self, byte_array: &[u8]) -> ManagedBuffer {
        let mut v: ArrayVec<u8, 30_000> = ArrayVec::new();
        for octet_array in byte_array.chunks(3) {
            v.extend(self.encode_chunks(octet_array));
        }

        let slice = v.as_slice();

        ManagedBuffer::from(slice)
    }

    fn encode_chunks(&self, chunks: &[u8]) -> ArrayVec<u8, 4> {
        let mut v = ArrayVec::new();
        match chunks.len() {
            3 => {
                v.push(LOOKUP_TABLE[self.extract_first_character_bits(chunks[0]) as usize]);
                v.push(
                    LOOKUP_TABLE[self.extract_second_character_bits(chunks[0], chunks[1]) as usize],
                );
                v.push(
                    LOOKUP_TABLE[self.extract_third_character_bits(chunks[1], chunks[2]) as usize],
                );
                v.push(LOOKUP_TABLE[(chunks[2] & 0b00111111) as usize]);
            }
            2 => {
                v.push(LOOKUP_TABLE[self.extract_first_character_bits(chunks[0]) as usize]);
                v.push(
                    LOOKUP_TABLE[self.extract_second_character_bits(chunks[0], chunks[1]) as usize],
                );
                v.push(LOOKUP_TABLE[self.extract_third_character_bits(chunks[1], 0) as usize]);
                v.push(PADDING);
            }
            1 => {
                v.push(LOOKUP_TABLE[self.extract_first_character_bits(chunks[0]) as usize]);
                v.push(LOOKUP_TABLE[self.extract_second_character_bits(chunks[0], 0) as usize]);
                v.push(PADDING);
                v.push(PADDING);
            }
            _ => {}
        }
        v
    }

    fn extract_first_character_bits(&self, first_byte: u8) -> u8 {
        (first_byte & 0b1111100) >> 2
    }

    fn extract_second_character_bits(&self, first_byte: u8, second_byte: u8) -> u8 {
        (first_byte & 0b00000011) << 4 | ((second_byte & 0b11110000) >> 4)
    }

    fn extract_third_character_bits(&self, second_byte: u8, third_byte: u8) -> u8 {
        (second_byte & 0b00001111) << 2 | ((third_byte & 0b11000000) >> 6)
    }

    fn generate_token_svg(&self, token: &EgldOrEsdtTokenIdentifier) -> ManagedBuffer {
        let mut final_buffer = ManagedBuffer::new_from_bytes(
            r##"<text x="149" y="860" font-size="18" class="J S T">"##.as_bytes(),
        );
        let token_buffer = token.clone().into_name();
        final_buffer.append(&token_buffer);
        final_buffer.append(&ManagedBuffer::new_from_bytes(b"</text>"));

        final_buffer
    }

    fn generate_cancelable_svg(&self, can_cancel: bool) -> ManagedBuffer {
        let mut final_buffer = ManagedBuffer::new_from_bytes(
            r##"<text x="416" y="860" font-size="18" class="J S T">"##.as_bytes(),
        );
        let can_cancel_buffer = if can_cancel {
            ManagedBuffer::new_from_bytes(b"Yes")
        } else {
            ManagedBuffer::new_from_bytes(b"No")
        };
        final_buffer.append(&can_cancel_buffer);
        final_buffer.append(&ManagedBuffer::new_from_bytes(b"</text>"));

        final_buffer
    }

    fn generate_duration_svg(&self, duration: u64) -> ManagedBuffer {
        let mut final_buffer = ManagedBuffer::new_from_bytes(
            r##"<text x="688" y="860" font-size="18" class="J S T">"##.as_bytes(),
        );

        let duration_buffer = if duration == 0 {
            ManagedBuffer::new_from_bytes(b"&lt; 1 Day")
        } else if duration == 1 {
            ManagedBuffer::new_from_bytes(b"1 Day")
        } else {
            let mut days_duration_buffer = self.u64_to_ascii(duration);
            days_duration_buffer.append_bytes(b" Days");
            days_duration_buffer
        };

        final_buffer.append(&duration_buffer);
        final_buffer.append(&ManagedBuffer::new_from_bytes(b"</text>"));

        final_buffer
    }

    fn generate_stream_id_svg(&self, stream_id: u64) -> ManagedBuffer {
        let mut final_buffer = ManagedBuffer::new_from_bytes(
            r##"<textPath startOffset="-90%" href="#A" class="J K N O"><animate additive="sum" attributeName="startOffset" begin="0s" dur="50s" from="0%" repeatCount="indefinite" to="100%"/>CoinDrip Protocol / Token Stream #"##.as_bytes()
        );

        let stream_id_buffer = self.u64_to_ascii(stream_id);
        final_buffer.append(&stream_id_buffer);
        final_buffer.append(&ManagedBuffer::new_from_bytes(r##"</textPath><textPath startOffset="10%" href="#A" class="J K N O"><animate additive="sum" attributeName="startOffset" begin="0s" dur="50s" from="0%" repeatCount="indefinite" to="100%"/>CoinDrip Protocol / Token Stream #"##.as_bytes()));
        final_buffer.append(&stream_id_buffer);
        final_buffer.append(&ManagedBuffer::new_from_bytes(b"</textPath>"));

        final_buffer
    }

    // TODO: Give credit for this function to Martin Wagner | CIO | Knights of Cathena
    fn u64_to_ascii(&self, number: u64) -> ManagedBuffer {
        let mut reversed_digits = ManagedVec::<Self::Api, u8>::new();
        let mut result = number.clone();

        while result > 0 {
            let digit = result % 10;
            result /= 10;

            let digit_char = match digit {
                0 => b'0',
                1 => b'1',
                2 => b'2',
                3 => b'3',
                4 => b'4',
                5 => b'5',
                6 => b'6',
                7 => b'7',
                8 => b'8',
                9 => b'9',
                _ => sc_panic!("invalid digit"),
            };

            reversed_digits.push(digit_char);
        }

        if &reversed_digits.len() == &0 {
            return ManagedBuffer::new_from_bytes(b"0");
        }

        let mut o = ManagedBuffer::new();

        for digit in reversed_digits.iter().rev() {
            o.append_bytes(&[digit]);
        }

        o
    }

    fn generate_svg(
        &self,
        token_svg: &ManagedBuffer,
        cancelable_svg: &ManagedBuffer,
        duration_svg: &ManagedBuffer,
        stream_id_svg: &ManagedBuffer,
    ) -> ManagedBuffer {
        let mut final_buffer = ManagedBuffer::new_from_bytes(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="1001" height="1000" fill="none"
            xmlns:v="https://vecta.io/nano">
            <style><![CDATA[.J{fill:#fff}.K{fill-opacity:.8}.L{fill:#0a1131}.M{stroke:#0a1131}.N{font-family:Courier New,Arial,monospace}.O{font-size:28px}.P{fill-opacity:.35}.Q{stroke-opacity:.2}.R{stroke-width:2}.S{font-family:Arial}.T{font-weight:bold}]]></style>
            <path fill="url(#B)" d="M0 0h1001v1000H0z"/>
            <rect x="74" y="73" width="853" height="853" rx="33.172" fill-opacity=".1" class="L"/>
            <rect x="76" y="75" width="849" height="849" rx="31.172" stroke-opacity=".15" stroke-width="4" class="M"/>
            <path d="M682.167 300.272L528.282 117.294l-49.504 41.445 154.534 183.75c30.517 36.287 44.452 81.851 40.291 129.409-8.57 97.955-95.202 170.484-193.376 161.895s-170.895-95.06-162.325-193.015c4.161-47.558 26.507-89.948 62.924-121.094l43.316-36.264 37.66 44.779-43.316 36.264c-24.752 20.723-39.455 49.477-42.187 80.709-5.713 65.303 42.984 123.208 108.433 128.934s123.462-42.843 129.175-108.146c2.795-31.942-6.691-62.812-27.531-86.808l-49.504 41.445c9.74 11.581 14.251 25.565 12.947 40.471-2.67 30.522-29.681 52.478-59.56 49.864-30.59-2.676-52.604-29.636-49.996-59.449 1.304-14.906 8.175-27.894 19.778-37.608l92.82-77.709-119.347-143.479-92.82 77.71c-49.504 41.445-80.456 100.248-86.045 164.132-11.613 132.736 87.73 250.861 220.763 262.499s251.378-87.44 262.991-220.176c5.589-63.884-14.032-126.396-56.236-176.58z" class="J"/>
            <rect x="124" y="791.015" width="206.965" height="92.879" rx="10.97" class="L P"/>
            <rect x="125" y="792.015" width="204.965" height="90.879" rx="9.97" class="M Q R"/>
            <rect x="396.785" y="791.015" width="206.965" height="92.879" rx="10.97" class="L P"/>
            <rect x="397.785" y="792.015" width="204.965" height="90.879" rx="9.97" class="M Q R"/>
            <rect x="669.57" y="791.015" width="206.965" height="92.879" rx="10.97" class="L P"/>
            <rect x="670.57" y="792.015" width="204.965" height="90.879" rx="9.97" class="M Q R"/>
            <path d="M152.192 822.593v-12.178h-4.35v-2.479h11.649v2.479h-4.34v12.178h-2.959zm8.239-7.239c0-1.493.223-2.746.669-3.759.334-.747.787-1.416 1.36-2.01.58-.593 1.213-1.033 1.9-1.32.913-.386 1.966-.579 3.16-.579 2.159 0 3.886.669 5.179 2.009 1.3 1.34 1.95 3.203 1.95 5.59 0 2.366-.644 4.219-1.93 5.559-1.287 1.333-3.006 1.999-5.159 1.999-2.18 0-3.913-.663-5.2-1.989-1.286-1.333-1.929-3.166-1.929-5.5zm3.049-.099c0 1.659.383 2.919 1.15 3.779.767.853 1.74 1.28 2.92 1.28s2.146-.423 2.899-1.27c.76-.853 1.14-2.13 1.14-3.829 0-1.68-.37-2.933-1.11-3.76-.733-.827-1.71-1.24-2.929-1.24s-2.203.42-2.95 1.26c-.747.833-1.12 2.093-1.12 3.78zm13.528 7.338v-14.657h2.96v6.509l5.979-6.509h3.979l-5.519 5.709 5.819 8.948h-3.829l-4.03-6.879-2.399 2.45v4.429h-2.96zm14.758 0v-14.657h10.869v2.479h-7.909v3.25h7.359v2.469h-7.359v3.99h8.189v2.469h-11.149zm13.688 0v-14.657h2.88l5.999 9.788v-9.788h2.75v14.657h-2.97l-5.909-9.558v9.558h-2.75zm219.75-5.389l2.87.91c-.44 1.6-1.173 2.79-2.2 3.57-1.02.773-2.316 1.159-3.889 1.159-1.947 0-3.546-.663-4.799-1.989-1.254-1.333-1.88-3.153-1.88-5.46 0-2.439.63-4.332 1.89-5.679 1.259-1.353 2.916-2.029 4.969-2.029 1.793 0 3.249.529 4.369 1.589.667.627 1.167 1.527 1.5 2.7l-2.93.7c-.173-.76-.536-1.36-1.089-1.8-.547-.44-1.214-.66-2-.66-1.087 0-1.97.39-2.65 1.17-.673.78-1.01 2.043-1.01 3.79 0 1.853.334 3.172 1 3.959s1.533 1.18 2.6 1.18c.786 0 1.463-.25 2.03-.75s.973-1.287 1.219-2.36zm5.5 5.389v-14.537h2.959v12.068h7.359v2.469h-10.318zm25.656 0h-3.219l-1.28-3.329h-5.859l-1.21 3.329h-3.14l5.709-14.657h3.13l5.869 14.657zm-5.449-5.799l-2.02-5.439-1.98 5.439h4zm6.939 5.799v-14.657h2.959v14.657h-2.959zm5.749 0v-14.657h4.429l2.66 9.998 2.63-9.998h4.439v14.657h-2.75v-11.538l-2.909 11.538h-2.85l-2.899-11.538v11.538h-2.75zm17.098 0v-14.657h10.868v2.479h-7.909v3.25h7.359v2.469h-7.359v3.99h8.189v2.469h-11.148zm13.648-14.657h5.409c1.22 0 2.15.093 2.79.279a4.91 4.91 0 0 1 2.209 1.35c.614.647 1.08 1.44 1.4 2.38.32.933.48 2.086.48 3.459 0 1.207-.15 2.247-.45 3.12-.366 1.067-.89 1.93-1.57 2.59-.513.5-1.206.89-2.079 1.17-.654.206-1.527.309-2.62.309h-5.569v-14.657zm2.959 2.479v9.709h2.21c.827 0 1.423-.047 1.79-.14.48-.12.876-.323 1.19-.61.32-.287.58-.757.78-1.41.2-.66.3-1.556.3-2.689s-.1-2.004-.3-2.61-.48-1.08-.84-1.42-.817-.57-1.37-.69c-.413-.093-1.223-.14-2.43-.14h-1.33zm191.297-2.479h5.409c1.22 0 2.15.093 2.79.279a4.91 4.91 0 0 1 2.209 1.35c.614.647 1.08 1.44 1.4 2.38.32.933.48 2.086.48 3.459 0 1.207-.15 2.247-.45 3.12-.366 1.067-.89 1.93-1.57 2.59-.513.5-1.206.89-2.079 1.17-.653.206-1.527.309-2.62.309h-5.569v-14.657zm2.959 2.479v9.709h2.21c.827 0 1.423-.047 1.79-.14.48-.12.876-.323 1.19-.61.32-.287.58-.757.78-1.41.2-.66.3-1.556.3-2.689s-.1-2.004-.3-2.61-.48-1.08-.84-1.42-.817-.57-1.37-.69c-.413-.093-1.223-.14-2.43-.14h-1.33zm11.829-2.479h2.959v7.938l.11 2.45c.127.6.427 1.083.9 1.45.48.36 1.133.54 1.96.54.84 0 1.473-.17 1.9-.51.426-.347.683-.77.77-1.27s.13-1.33.13-2.49v-8.108h2.959v7.698c0 1.76-.08 3.003-.24 3.73s-.456 1.34-.89 1.84c-.426.5-1 .9-1.719 1.2-.72.293-1.66.439-2.82.439-1.4 0-2.463-.16-3.19-.479-.719-.327-1.289-.747-1.709-1.26-.42-.52-.697-1.063-.83-1.63-.194-.84-.29-2.08-.29-3.72v-7.818zm14.828 14.657v-14.657h6.229c1.566 0 2.703.133 3.409.399.714.26 1.284.727 1.71 1.4s.64 1.443.64 2.31c0 1.1-.323 2.01-.97 2.73-.646.713-1.613 1.163-2.899 1.349.64.374 1.166.784 1.579 1.23.42.447.984 1.24 1.69 2.38l1.79 2.859h-3.54l-2.139-3.189-1.56-2.15c-.28-.3-.577-.503-.89-.61-.313-.113-.81-.17-1.49-.17h-.6v6.119h-2.959zm2.959-8.458h2.19c1.42 0 2.306-.06 2.66-.18s.63-.327.83-.62.3-.66.3-1.1c0-.493-.134-.89-.4-1.19-.26-.307-.63-.5-1.11-.58-.24-.033-.96-.05-2.16-.05h-2.31v3.72zm25.047 8.458h-3.22l-1.28-3.329h-5.859l-1.21 3.329h-3.139l5.709-14.657h3.13l5.869 14.657zm-5.449-5.799l-2.02-5.439-1.98 5.439h4zm8.808 5.799v-12.178h-4.349v-2.479h11.648v2.479h-4.339v12.178h-2.96zm9.129 0v-14.657h2.96v14.657h-2.96zm5.189-7.239c0-1.493.224-2.746.67-3.759.334-.747.787-1.416 1.36-2.01.58-.593 1.213-1.033 1.9-1.32.913-.386 1.966-.579 3.159-.579 2.16 0 3.887.669 5.18 2.009 1.3 1.34 1.949 3.203 1.949 5.59 0 2.366-.643 4.219-1.929 5.559-1.287 1.333-3.007 1.999-5.16 1.999-2.179 0-3.912-.663-5.199-1.989-1.286-1.333-1.93-3.166-1.93-5.5zm3.05-.099c0 1.659.383 2.919 1.15 3.779a3.75 3.75 0 0 0 2.919 1.28c1.18 0 2.147-.423 2.9-1.27.76-.853 1.14-2.13 1.14-3.829 0-1.68-.37-2.933-1.11-3.76-.733-.827-1.71-1.24-2.93-1.24s-2.203.42-2.949 1.26c-.747.833-1.12 2.093-1.12 3.78zm13.518 7.338v-14.657h2.88l5.999 9.788v-9.788h2.749v14.657h-2.969l-5.909-9.558v9.558h-2.75z" class="J K"/>"##.as_bytes(),
        );

        final_buffer.append(token_svg);
        final_buffer.append(cancelable_svg);
        final_buffer.append(duration_svg);

        final_buffer.append_bytes(r##"<path id="A" fill="none" d="M125 45h750s80 0 80 80v750s0 80-80 80H125s-80 0-80-80V125s0-80 80-80"/>
        <defs>
            <linearGradient id="B" x1="233.15" y1="37.5" x2="739.158" y2="887.945" gradientUnits="userSpaceOnUse">
                <stop stop-color="#2563eb"/>
                <stop offset="1" stop-color="#1e2e75"/>
            </linearGradient>
        </defs>
        <text text-rendering="optimizeSpeed">"##.as_bytes());

        final_buffer.append(stream_id_svg);

        final_buffer.append_bytes(r##"</text></svg>"##.as_bytes());

        final_buffer
    }

    fn generate_svg_from_stream(&self, stream: &Stream<Self::Api>) -> ManagedBuffer {
        let token_svg = self.generate_token_svg(&stream.payment_token);
        let cancelable_svg = self.generate_cancelable_svg(stream.can_cancel);

        let stream_duration_seconds = stream.end_time - stream.start_time;
        let stream_duration_days = stream_duration_seconds / 60 / 60 / 24;
        let duration_svg = self.generate_duration_svg(stream_duration_days);

        let stream_id_svg = self.generate_stream_id_svg(stream.id);

        let svg = self.generate_svg(&token_svg, &cancelable_svg, &duration_svg, &stream_id_svg);

        let mut arr: [u8; 30_000] = [0; 30_000];
        let new_arr = svg.load_to_byte_array(&mut arr);

        let svg_base64 = self.base64(new_arr);

        svg_base64
    }
}
