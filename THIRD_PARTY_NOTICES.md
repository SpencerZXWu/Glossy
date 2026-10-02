# Third-party notices

Glossy itself is MIT licensed — see [LICENSE](./LICENSE). What it reads and runs
with is other people's work, and this file records whose it is, where it came
from and under what terms. The same list, with the sizes the app shows before it
downloads anything, is on the **Resources** page in the settings window.

Nothing here ships inside the installer. Every file below is downloaded when the
user asks for it, straight from the address listed with it, and checked against
the digest recorded here before it is used. Passing these files on is therefore
not something this project does: it is what the user's own machine does when a
download button is pressed. The notices are kept anyway, because that is what
the licences ask of anyone who handles the files at all.

## Text recognition on this machine

The recogniser, its dictionary and the detector are **PaddleOCR's PP-OCR
models** (the `mobile` models of `PP-OCRv4`), distributed as ONNX by
**RapidOCR** at <https://www.modelscope.cn/models/RapidAI/RapidOCR>.

- Copyright: © Baidu and/or the applicable PaddleOCR rights holders.
- Licence: **Apache License 2.0**, whose text is below.
- RapidOCR states this in `python/MODEL_LICENSES.md` in
  <https://github.com/RapidAI/RapidOCR>: the model artifacts are Apache-2.0 and
  "these permissions are not restricted to non-commercial use". Converting a
  model to ONNX does not change its terms, and these models are used here
  unmodified.
- Apache-2.0 grants no right to the PaddleOCR or Baidu names or marks. Glossy is
  not affiliated with, endorsed by or sponsored by either of them.

### The files

- `onnxruntime.dll` — 17,766,712 bytes — SHA-256 `14e186627f109a15f28f0b36f6e8ea07c79eac666818e63c39a9792f415bf649`
  <https://files.pythonhosted.org/packages/9f/10/3d946d5d5f2cdcc3c8da36cae63190c516d16349edaffd944bda60ca4c3e/onnxruntime-1.28.0-cp311-cp311-win_amd64.whl>
  the library is taken out of this wheel
- `ch_PP-OCRv4_det_mobile.onnx` — 4,745,517 bytes — SHA-256 `d2a7720d45a54257208b1e13e36a8479894cb74155a5efe29462512d42f49da9`
  <https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=onnx/PP-OCRv4/det/ch_PP-OCRv4_det_mobile.onnx>
- `ppocr_keys_v1.txt` — 26,249 bytes — SHA-256 `28b2362ad4ab2dc38769aa72feb535e3a9ddb3fd2a7585a05920e6393b1dc7f7`
  <https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=paddle/PP-OCRv4/rec/ch_PP-OCRv4_rec_mobile/ppocr_keys_v1.txt>
- `ch_PP-OCRv4_rec_mobile.onnx` — 10,857,958 bytes — SHA-256 `48fc40f24f6d2a207a2b1091d3437eb3cc3eb6b676dc3ef9c37384005483683b`
  <https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=onnx/PP-OCRv4/rec/ch_PP-OCRv4_rec_mobile.onnx>
- `japan_PP-OCRv4_rec_mobile.onnx` — 9,753,335 bytes — SHA-256 `e1075a67dba758ecfc7ebc78a10ae61c95ac8fb66a9c86fab5541e33f085cb7a`
  <https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=onnx/PP-OCRv4/rec/japan_PP-OCRv4_rec_mobile.onnx>
- `japan_dict.txt` — 17,332 bytes — SHA-256 `1dcfcb41eec90576a945b3084f22ade11ced506e24f14879245b071698f308e8`
  <https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=paddle/PP-OCRv4/rec/japan_PP-OCRv4_rec_mobile/japan_dict.txt>
- `chinese_cht_PP-OCRv3_rec_mobile.onnx` — 11,152,536 bytes — SHA-256 `779656d044ce388045e02ea9244724616194e63928606436cdfc6dc3c9528cc6`
  <https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=onnx/PP-OCRv4/rec/chinese_cht_PP-OCRv3_rec_mobile.onnx>
- `chinese_cht_dict.txt` — 33,443 bytes — SHA-256 `832551fee1f2fbc97508772d81ebdc8dba12c00de97a35c71c9ddf43ddac1a83`
  <https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=paddle/PP-OCRv4/rec/chinese_cht_PP-OCRv3_rec_mobile/chinese_cht_dict.txt>
- `latin_PP-OCRv3_rec_mobile.onnx` — 8,978,191 bytes — SHA-256 `e9d7a33667e8aaa702862975186adf2012e3f390cc0f9422865957125f8071cf`
  <https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=onnx/PP-OCRv4/rec/latin_PP-OCRv3_rec_mobile.onnx>
- `latin_dict.txt` — 468 bytes — SHA-256 `8e6d4e3629788c35c31f7e530287d6147b549bb7a265bd6708bb281134429e2c`
  <https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=paddle/PP-OCRv4/rec/latin_PP-OCRv3_rec_mobile/latin_dict.txt>
- `cyrillic_PP-OCRv3_rec_mobile.onnx` — 8,972,413 bytes — SHA-256 `1efb65bdc460af1c0e8733d005b20952b17ca5aac10ddb56c968333791c5eaa3`
  <https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=onnx/PP-OCRv4/rec/cyrillic_PP-OCRv3_rec_mobile.onnx>
- `cyrillic_dict.txt` — 410 bytes — SHA-256 `369a82c6c8c479784a5d726448b83b1eafb5fef0a4129a5eaa3929625ddcd132`
  <https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=paddle/PP-OCRv4/rec/cyrillic_PP-OCRv3_rec_mobile/cyrillic_dict.txt>
- `korean_PP-OCRv4_rec_mobile.onnx` — 24,067,780 bytes — SHA-256 `ab151ba9065eccd98f884cf4d927db091be86137276392072edd4f9d43ad7426`
  <https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=onnx/PP-OCRv4/rec/korean_PP-OCRv4_rec_mobile.onnx>
- `korean_dict.txt` — 14,480 bytes — SHA-256 `aa1fdc8ae8f7cd40a0ec4edb472eb0421e11427e6ccfee9915440742c18b0a20`
  <https://www.modelscope.cn/api/v1/models/RapidAI/RapidOCR/repo?Revision=master&FilePath=paddle/PP-OCRv4/rec/korean_PP-OCRv4_rec_mobile/korean_dict.txt>

## Offline translation

The two translation directions are **OPUS-MT**'s `opus-mt-en-zh` and
`opus-mt-zh-en`, published by **Helsinki-NLP** at
<https://huggingface.co/Helsinki-NLP/opus-mt-en-zh> and
<https://huggingface.co/Helsinki-NLP/opus-mt-zh-en>, and exported to ONNX by
**Xenova** at <https://huggingface.co/Xenova/opus-mt-en-zh> and
<https://huggingface.co/Xenova/opus-mt-zh-en>. They are Marian models: six
encoder and six decoder layers trained on the OPUS corpus, and the files
downloaded here are the int8 quantization of those exports.

The two are **not under the same licence**, which is why they are named apart:

- `opus-mt-en-zh`, the English into Chinese direction — **Apache License 2.0**,
  whose text is in this file below.
- `opus-mt-zh-en`, the Chinese into English direction — **Creative Commons
  Attribution 4.0 International** (CC-BY-4.0), which asks that the source be
  named — which is what this section does. Commercial use is not restricted by
  it.

Both directions carry © the Helsinki-NLP group at the University of Helsinki and
the OPUS project's contributors. The models are used unmodified apart from the
quantization they were published with; nothing here changes their terms.

### The files

- `zh-en_encoder.onnx` — 52,899,742 bytes — SHA-256 `84d5e171b626bc8b6b220d022ac58696e9528c25deeacca62b5cbf4364547a99`
  <https://www.modelscope.cn/api/v1/models/Xenova/opus-mt-zh-en/repo?Revision=master&FilePath=onnx/encoder_model_quantized.onnx>
- `zh-en_decoder.onnx` — 59,842,102 bytes — SHA-256 `debcc3054b9ff9aaed972bd6e251a459abe1e2f1402da1efa63163dc2f46ef73`
  <https://www.modelscope.cn/api/v1/models/Xenova/opus-mt-zh-en/repo?Revision=master&FilePath=onnx/decoder_model_quantized.onnx>
- `zh-en_tokenizer.json` — 6,381,339 bytes — SHA-256 `b306d0301cf280bfd647d7067b5ade2a97b987e6d678df110703c002433643ff`
  <https://www.modelscope.cn/api/v1/models/Xenova/opus-mt-zh-en/repo?Revision=master&FilePath=tokenizer.json>
- `en-zh_encoder.onnx` — 52,899,742 bytes — SHA-256 `d3b7912bf6a9bd27e4c074c2df91d4ff3d5b4bc5f7f6c8d7cc9c805c98fbafee`
  <https://www.modelscope.cn/api/v1/models/Xenova/opus-mt-en-zh/repo?Revision=master&FilePath=onnx/encoder_model_quantized.onnx>
- `en-zh_decoder.onnx` — 59,842,102 bytes — SHA-256 `2c66a3981099b40edbb3a0d65e015d05964d42fecb9780f8422776eff5939112`
  <https://www.modelscope.cn/api/v1/models/Xenova/opus-mt-en-zh/repo?Revision=master&FilePath=onnx/decoder_model_quantized.onnx>
- `en-zh_tokenizer.json` — 6,380,952 bytes — SHA-256 `d0c7da27056e8f42adce9e76d8e792e5daa64e15f5acd2e7aabf0121877dd4c1`
  <https://www.modelscope.cn/api/v1/models/Xenova/opus-mt-en-zh/repo?Revision=master&FilePath=tokenizer.json>

The addresses are the mirror `modelscope.cn` rather than `huggingface.co`: the
files are the same ones, byte for byte, and the mirror is the one that answers
without a proxy on a mainland connection.

## ONNX Runtime

Recognition runs on **ONNX Runtime**, © Microsoft Corporation, under the **MIT
License** whose text is below. The library is taken out of the wheel published
on PyPI rather than compiled here:

- Wheel: <https://pypi.org/project/onnxruntime/1.28.0/>
- File used: `onnxruntime/capi/onnxruntime.dll` from that wheel.

## Everything else

Glossy itself is MIT licensed. The crates it is built from are recorded in
`src-tauri/Cargo.lock` with their own licences and are not covered by this file;
nothing from them is reproduced here.

## Licence texts

### Apache License 2.0


                                 Apache License
                           Version 2.0, January 2004
                        http://www.apache.org/licenses/

   TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

   1. Definitions.

      "License" shall mean the terms and conditions for use, reproduction,
      and distribution as defined by Sections 1 through 9 of this document.

      "Licensor" shall mean the copyright owner or entity authorized by
      the copyright owner that is granting the License.

      "Legal Entity" shall mean the union of the acting entity and all
      other entities that control, are controlled by, or are under common
      control with that entity. For the purposes of this definition,
      "control" means (i) the power, direct or indirect, to cause the
      direction or management of such entity, whether by contract or
      otherwise, or (ii) ownership of fifty percent (50%) or more of the
      outstanding shares, or (iii) beneficial ownership of such entity.

      "You" (or "Your") shall mean an individual or Legal Entity
      exercising permissions granted by this License.

      "Source" form shall mean the preferred form for making modifications,
      including but not limited to software source code, documentation
      source, and configuration files.

      "Object" form shall mean any form resulting from mechanical
      transformation or translation of a Source form, including but
      not limited to compiled object code, generated documentation,
      and conversions to other media types.

      "Work" shall mean the work of authorship, whether in Source or
      Object form, made available under the License, as indicated by a
      copyright notice that is included in or attached to the work
      (an example is provided in the Appendix below).

      "Derivative Works" shall mean any work, whether in Source or Object
      form, that is based on (or derived from) the Work and for which the
      editorial revisions, annotations, elaborations, or other modifications
      represent, as a whole, an original work of authorship. For the purposes
      of this License, Derivative Works shall not include works that remain
      separable from, or merely link (or bind by name) to the interfaces of,
      the Work and Derivative Works thereof.

      "Contribution" shall mean any work of authorship, including
      the original version of the Work and any modifications or additions
      to that Work or Derivative Works thereof, that is intentionally
      submitted to Licensor for inclusion in the Work by the copyright owner
      or by an individual or Legal Entity authorized to submit on behalf of
      the copyright owner. For the purposes of this definition, "submitted"
      means any form of electronic, verbal, or written communication sent
      to the Licensor or its representatives, including but not limited to
      communication on electronic mailing lists, source code control systems,
      and issue tracking systems that are managed by, or on behalf of, the
      Licensor for the purpose of discussing and improving the Work, but
      excluding communication that is conspicuously marked or otherwise
      designated in writing by the copyright owner as "Not a Contribution."

      "Contributor" shall mean Licensor and any individual or Legal Entity
      on behalf of whom a Contribution has been received by Licensor and
      subsequently incorporated within the Work.

   2. Grant of Copyright License. Subject to the terms and conditions of
      this License, each Contributor hereby grants to You a perpetual,
      worldwide, non-exclusive, no-charge, royalty-free, irrevocable
      copyright license to reproduce, prepare Derivative Works of,
      publicly display, publicly perform, sublicense, and distribute the
      Work and such Derivative Works in Source or Object form.

   3. Grant of Patent License. Subject to the terms and conditions of
      this License, each Contributor hereby grants to You a perpetual,
      worldwide, non-exclusive, no-charge, royalty-free, irrevocable
      (except as stated in this section) patent license to make, have made,
      use, offer to sell, sell, import, and otherwise transfer the Work,
      where such license applies only to those patent claims licensable
      by such Contributor that are necessarily infringed by their
      Contribution(s) alone or by combination of their Contribution(s)
      with the Work to which such Contribution(s) was submitted. If You
      institute patent litigation against any entity (including a
      cross-claim or counterclaim in a lawsuit) alleging that the Work
      or a Contribution incorporated within the Work constitutes direct
      or contributory patent infringement, then any patent licenses
      granted to You under this License for that Work shall terminate
      as of the date such litigation is filed.

   4. Redistribution. You may reproduce and distribute copies of the
      Work or Derivative Works thereof in any medium, with or without
      modifications, and in Source or Object form, provided that You
      meet the following conditions:

      (a) You must give any other recipients of the Work or
          Derivative Works a copy of this License; and

      (b) You must cause any modified files to carry prominent notices
          stating that You changed the files; and

      (c) You must retain, in the Source form of any Derivative Works
          that You distribute, all copyright, patent, trademark, and
          attribution notices from the Source form of the Work,
          excluding those notices that do not pertain to any part of
          the Derivative Works; and

      (d) If the Work includes a "NOTICE" text file as part of its
          distribution, then any Derivative Works that You distribute must
          include a readable copy of the attribution notices contained
          within such NOTICE file, excluding those notices that do not
          pertain to any part of the Derivative Works, in at least one
          of the following places: within a NOTICE text file distributed
          as part of the Derivative Works; within the Source form or
          documentation, if provided along with the Derivative Works; or,
          within a display generated by the Derivative Works, if and
          wherever such third-party notices normally appear. The contents
          of the NOTICE file are for informational purposes only and
          do not modify the License. You may add Your own attribution
          notices within Derivative Works that You distribute, alongside
          or as an addendum to the NOTICE text from the Work, provided
          that such additional attribution notices cannot be construed
          as modifying the License.

      You may add Your own copyright statement to Your modifications and
      may provide additional or different license terms and conditions
      for use, reproduction, or distribution of Your modifications, or
      for any such Derivative Works as a whole, provided Your use,
      reproduction, and distribution of the Work otherwise complies with
      the conditions stated in this License.

   5. Submission of Contributions. Unless You explicitly state otherwise,
      any Contribution intentionally submitted for inclusion in the Work
      by You to the Licensor shall be under the terms and conditions of
      this License, without any additional terms or conditions.
      Notwithstanding the above, nothing herein shall supersede or modify
      the terms of any separate license agreement you may have executed
      with Licensor regarding such Contributions.

   6. Trademarks. This License does not grant permission to use the trade
      names, trademarks, service marks, or product names of the Licensor,
      except as required for reasonable and customary use in describing the
      origin of the Work and reproducing the content of the NOTICE file.

   7. Disclaimer of Warranty. Unless required by applicable law or
      agreed to in writing, Licensor provides the Work (and each
      Contributor provides its Contributions) on an "AS IS" BASIS,
      WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
      implied, including, without limitation, any warranties or conditions
      of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A
      PARTICULAR PURPOSE. You are solely responsible for determining the
      appropriateness of using or redistributing the Work and assume any
      risks associated with Your exercise of permissions under this License.

   8. Limitation of Liability. In no event and under no legal theory,
      whether in tort (including negligence), contract, or otherwise,
      unless required by applicable law (such as deliberate and grossly
      negligent acts) or agreed to in writing, shall any Contributor be
      liable to You for damages, including any direct, indirect, special,
      incidental, or consequential damages of any character arising as a
      result of this License or out of the use or inability to use the
      Work (including but not limited to damages for loss of goodwill,
      work stoppage, computer failure or malfunction, or any and all
      other commercial damages or losses), even if such Contributor
      has been advised of the possibility of such damages.

   9. Accepting Warranty or Additional Liability. While redistributing
      the Work or Derivative Works thereof, You may choose to offer,
      and charge a fee for, acceptance of support, warranty, indemnity,
      or other liability obligations and/or rights consistent with this
      License. However, in accepting such obligations, You may act only
      on Your own behalf and on Your sole responsibility, not on behalf
      of any other Contributor, and only if You agree to indemnify,
      defend, and hold each Contributor harmless for any liability
      incurred by, or claims asserted against, such Contributor by reason
      of your accepting any such warranty or additional liability.

   END OF TERMS AND CONDITIONS

   APPENDIX: How to apply the Apache License to your work.

      To apply the Apache License to your work, attach the following
      boilerplate notice, with the fields enclosed by brackets "[]"
      replaced with your own identifying information. (Don't include
      the brackets!)  The text should be enclosed in the appropriate
      comment syntax for the file format. We also recommend that a
      file or class name and description of purpose be included on the
      same "printed page" as the copyright notice for easier
      identification within third-party archives.

   Copyright [yyyy] [name of copyright owner]

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.

### MIT License (ONNX Runtime)

MIT License

Copyright (c) Microsoft Corporation

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

### Creative Commons Attribution 4.0 International

The `opus-mt-zh-en` model is licensed under CC-BY-4.0, whose full legal code is at
<https://creativecommons.org/licenses/by/4.0/legalcode>. The `opus-mt-en-zh` model is
under the Apache License 2.0 above instead. In short:

You are free to share (copy and redistribute the material in any medium or
format) and to adapt (remix, transform, and build upon the material) for any
purpose, even commercially. This is on the condition that you give appropriate
credit, provide a link to the licence, and indicate if changes were made — which
is what the section above does — and that you do not apply legal terms or
technological measures that legally restrict others from doing anything the
licence permits. No warranties are given; the licence may not give you all of
the permissions necessary for your intended use.

