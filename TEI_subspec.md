# TEI subspace used for critic

## This Document
This document is written as a guideline for the creation and use of TEI documents employed in `critic`.
Some aspects of this document refer not specifically to TEI encoding, but broader aspects of `critic`, like its file structure.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED",  "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119).

# File Layout
Each Manuscript in its entirety MUST be inside a single TEI file.
## File Naming Scheme
The names for TEI files SHOULD be a unique name of the manuscript.

## Internal files for single pages
When it is desirable to only give transcription information about a single page, the `<div type="page"/>` alone MAY be put in a single file.
It should be noted that this file *is not* valid TEI since it misses the header.

When only giving information about a single page, a full TEI file with header MAY NOT be used. This restriction is to prevent the header information being duplicated across several files.

# Metadata
## The TEI Header
This TEI Header (`<teiHeader>`) MUST be present in every TEI file.

### titleStmt
The `title` inside `titleStmt` MUST be given as `{manuscript name}`.

### publicationStmt
The `publicationStmt` MUST be given as `<p>This digital reproduction is published as part of TanakhCC and licensed as https://creativecommons.org/publicdomain/zero/1.0.</p>`.
In particular, all data that is part of TanakhCC MUST always explicitly be CC0.

### sourceDesc
The `sourceDesc` MUST be given and contain exactly one element `msDesc`, defined in the next subsection.

#### Manuscript Description {#msDesc}
- The Body of each file MUST begin with a `msDesc`.
- The `msDesc` MUST have
    - `msIdentifier`, defining the physical manuscript reproduced:
        - `institution` SHOULD be given if relevant
        - `collection` MAY be given if relevant
        - `altIdentifier` MAY be given any number of times, each containing a single `idno`, which contains an alternative identifier for this MS
            - For Example, for Codex `S1`, you may add an `altIdentifier` each for `Safra, JUD002`, and `Sassoon 1053`
- `physDesc` MUST be given and MAY contain `handDesc` and `scriptDesc` to describe the characteristics of scribal hands or the script used.

# Representing the Text itself
As a general rule of thumb, our goal is to reproduce the physical text as closely as possible.
If a reconstruction is not obvious, it is preferable to skip it; otherwise we risk corrupting the datapool with reconstructions from other text types.
Reconstructions SHOULD be done in the following cases:
- When characters are difficult but possible to read.
Otherwise, reconstructions MAY be done, but SHOULD be avoided, unless their certainty is high.
Reconstructions MUST NOT be done to correct scribal errors, including but not limited to:
- incorrect orthography
- homoioteleuta / -arkta
- skipped words, even when alliterations or other markers make a genuine mistake probable
All of the above phenomena MUST be represented without emandation. See the later sections for the correct markup to use.

## Text Structure
Text is structured as three levels of nested divs:
- page (represented by a `div` with `type="page"`)
    - column (represented by a `div` with `type="column"`)
        - line (represented by a `div` with `type="line"`)
        - Every block of normal text MUST be enclosed in `<p>` unless it is special in some way (see below for special cases).
For the page-level, the `n` attribute MUST be given and contain the name of the page.
Page names SHOULD be a sequential number, or `{folio-nr}_{r/v}` for folios.
In any case, page names MUST be in lexical order (i.e. a page appearing first in the reading order of the MS MUST have a lexically smaller name).
Pages now missing from the MS SHOULD be given as an empty `<div type="page" n="{pagename}/>`.

For column and line, the `n` attribute with the correct number SHOULD be given.
If it is not given, the number is supplied sequentially, starting from 1.

## Defining the source Language {#Defining-the-source-language}
- The source language MUST be defined for each part of the transcription.
- The source language MUST be defined in the `xml:lang` attribute.
- The source language SHOULD be defined on the highest level possible, up to the `body` tag.
    - When there are different source languages given at different depths, the first language encountered when traversing the manuscript upwards is used.
- BCP47 Codes MUST be used to define the source language.
    - Common Codes used for the transcription of biblical manuscripts are:
        - `hbo-Hebr`, `hbo-Phnx`, `smp-Hebr`, `grc`

### Versification
It is the goal of text criticism to be as faithful to each manuscript as possible, and to approach ancient texts on their own merit.
Versification is very convenient for everyday use, but it is my opinion that faithfulness to the witness beats convenience.
Therefore, physical structure (layout in columns and lines) is given priority over logical structure (versification).
Manuscripts that have verse breaks (particularly: sof passuqim in MT manuscripts) SHOULD be transcribed - they are physically extent artefacts, so we want to aknowledge them.

Whenever verse breaks are:
- physically apperent (sof passuqim, line indentation, actual numbering, ...)
OR
- reconstructable with certainty (comparison with other MSS or standard versions)
then the beginnings of verses SHOULD be indicated by `<anchor>` tags.
The `<anchor>` tag...
- MUST include `type="{the-versification-scheme}"`.
- MUST have `xml:id="A_V_{the-versification-scheme-shorthand}_{the-verse-ID}"`.

#### Considerations when adding Verse anchors
Verse anchors are used to align texts in the collation phase. This means that wrong verse anchors will lead to wrong collation and all sorts of Problems.
The strict rule is therefore: When Verse boundaries are uncertain (e.g. words moving between two adjacent verses in different versions),
verse anchors MUST NOT be used.

On the other hand, when different versions have text rearranged or differences in verse numbers and the verse boundaries are certain, it is RECOMMENDED that
verse anchors be supplied in as many versions as possible. This aids collation and indexing of the data.

#### Available Versification schemes
In ALL cases, the abbreviation for books from [OSIS](https://wiki.crosswire.org/OSIS_Book_Abbreviations) MUST be used.
While this may use book abbreviations foreign to some(or most) schemes, this simplifies crossreferencing drasticly.

Verse IDs in all schemes are formed as:
`{book_abbr}-{chapter}-{verse}`

##### Common
Shorthand: `C`
Long Form: `Common`

When all or most versification schemes aggree, you may choose to use the `Common` scheme instead of entering the same verse number in all the different schemes.
This scheme SHOULD be used, when all of the following schemes agree:
- Masoretic
- Septuagint
- ESV
- Present
When the above schemes do not agree, the `Common` scheme MUST NOT be used.
If other schemes disagree on the verse boundary in question, their anchors MAY be given, but it is NOT RECOMMENDED to do so.
Instead, disagreement for uncommon versification schemes is best noted in a manuscript for which the uncommon scheme is the natural versification scheme.
By way of example: when Aleppo disagrees with BHS, a septuagintal manuscript SHOULD NOT add an anchor for the Aleppo versification.

##### Present
Shorthand: `P`
Long Form: `Present`

The verse numbering (explicit or implicit, by counting from the chapter starts) used in the present manuscript.

##### Masoretic (BHS)
Shorthand: `MT`
Long Form: `Masoretic`

The Mainline Masoretic Versification Scheme is that used by the BHS.

##### Masoretic (Aleppo)
Shorthand: `MT-Aleppo`
Long Form: `Masoretic-Aleppo`

The versification used in the Aleppo Codex.

##### Septuagint (???)
TODO: prb Rahlfs???

##### Vulgata

##### Samaritan Pentateuch

##### English Standard Version
Shorthand: `ESV`
Long Form: `ESV`

The versification used in the ESV, 2016 Edition. This Versification scheme is included, since it is commonly used by modern sources.

## Abbreviations
When a sequence abbreviates another sequence:
- The entire part of text MUST be enclosed in `<p><choice>`
- The surface form MUST be given in `<abbr>`
- The (conjectured) expanded form MUST be given in `<expan>`

## Ligatures
TODO: just use gaiji to represent them as glyphs. Use abbreviation as before for NS and the like

## Damaged Characters {#uncertain}
This occurs where characters are present, but physically damaged.
- When a reconstruction is too difficult to make with any certainty, use [Lacunous Elements](#lacuna) instead.
- This section does NOT apply when characters are impossible to decipher from their remaining artefacts alone. Reconstructions based on grammar, common words or phrases alone MUST NOT be made. Use [Lacunous Elements](#lacuna) instead.

The damaged characters MUST be enclosed in `<p><damaged>`.
- `@agent` SHOULD be given
    - good examples of reasons include: `smeared`, `burned`, `water`
    - the `@agent` MAY NOT contain whitespace
    - If a space would be desired to split words (e.g. `smeared diacritica`), `-` SHOULD be used instead (`smeared-diacritica`)
- `@cert` SHOULD be given

## Lacunous Elements {#lacuna}
This occurs when the existance of characters, lines, columns etc is obvious (for grammatical-, layout- or other reasons), but they are illegible. Whether physical traces of these characters such as illegible smears of ink are present or not is irrelevant. If the characters are hard but possible to read, use [Damaged Characters](#uncertain) instead.

Instead of missing characters, a `<gap>` element MUST be used, directly inside the lines `<div>`.
- `@reason` MUST be given
    - good examples of reasons include: `lost`, `smeared`, `burned`, `water`
- `@unit` MUST be given and MUST be one of `character`, `line`, `column`
- `@n` MUST be given
- `@cert` MAY be given and qualifies both the certainty in assertaining the amount of missing units as well as the proposed reconstruction if any

Missing pages MUST NOT be marked as lacuna. Instead, they SHOULD be marked by adding an empty `<div type="page" n="{pagename}"/>`.

## Ancient Corrections
When multiple ancient surface forms are present in a place, these rules apply.
Versions are here considered in (conjectured) temporal order.
So when a word was written by Scribe1, struck through by Scribe2 and then another word written atop it by Scribe3, there are three distinct versions.
- All different ancient versions (the first, and all different corrections) MUST be given separately.
- The entire passage in question MUST be wrapped in `<app>`.
- Every individual version MUST be given in `<rdg>`.
- `@hand` SHOULD be given for all `<rdg>`s.
- `@varSeq` MUST be given for all `<rdg>`s and mark them in (conjectured) temporal order. `@varSeq` MUST NOT contain additional semantic information. Normalization MAY turn it into zero-based indices.
- The text in `<rdg>` MUST be normal text without additional markup elements.

If part of the text in any of the versions is lacunous, exclude it from its reading. Instead, add a lacuna after the entire correction.
If part of the text in any of the versions is damaged but legible, simply add it without markup.

## Significant space
When a significant space is left in the ancient manuscript, this MUST be recorded using the `<space>` element.
- normal spacing between words MUST NOT be recorded
- space MUST be recorded if it is both (1) large enough to fit two full-width-characters (i.e. ignoring punctuation or `י`) and (2) the normal word-spacing of the manuscript is smaller then this space.
- space MUST be recorded if it spans an entire line or more
- the `@quantity` MUST be given
- `@unit` MUST be given and MUST be one of `character`, `line`, `column`

## Nonstandard Glyphs and Diacritica
### Non-Tiberian Niqud
For vocalization that is not tiberian, you MUST transcribe the vowels with the equivalent tiberian niqud.
An enclosing element (see above, [Source Language](#Defining-the-source-language)) must carry the language tag with the following tags:
- `hbo-Hebr-x-babli-e`
- `hbo-Hebr-x-babli-k`
- `hbo-Hebr-x-palest`

