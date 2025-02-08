# TEI subspace used for critic

## This Document
This document is written as a guideline for the creation and use of TEI documents employed in `critic`.
Some aspects of this document refer not specifically to TEI encoding, but broader aspects of `critic`, like its file structure.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED",  "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119).

# File Layout
Each page (not folio) of an ancient manuscript MUST reside in a separate TEI file.
## File Naming Scheme
The names for TEI files SHOULD be `{manuscript name}_{folio}_{recto/verso}` for folios.

Manuscripts in scroll form (or other forms without natural pagebreaks) SHOULD be given as a single TEI file.
Their filename SHOULD be `{manuscript name}`

## Directory Structure
`critic` uses this directory structure. For interchange purposes, you are free to use any directory structure that seems convenient.
- `{manuscript name}`
    - page files

# Metadata
## The TEI Header
This TEI Header MUST be present in every individual TEI file.

### titleStmt
The `title` inside `titleStmt` MUST be given as `{manuscript name} folio {folio number} {recto/verse}.`

### publicationStmt
The `publicationStmt` MUST be given as `This digital reproduction is published as part of TanakhCC and licensed as CC0 1.0.`.
In particular, all data that is part of TanakhCC MUST always explicitly be CC0.

### sourceDesc
The `sourceDesc` MUST be given and contain exactly one element `msDesc`, defined in the next subsection.

#### Manuscript Description
- The Body of each file MUST begin with a `msDesc`.
- The `msDesc` MUST have `msIdentifier`, defining the physical manuscript reproduced.
    - `institution` and `msName` SHOULD be given if relevant
    - `collection` MAY be given if relevant
- `physDesc` MAY be given and contain `handDesc` and `scriptDesc` to describe the characteristics of scribal hands or the script used.

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

## Defining the source Language
- The source language MUST be defined for each part of the transcription.
- The source language MUST be defined in the `xml:lang` attribute.
- The source language SHOULD be defined on the highest level possible, up to the `body` tag.
    - When there are different source languages given at different depths, the first language encountered when traversing the manuscript upwards is used.
- BCP47 Codes MUST be used to define the source language.
    - Common Codes used for the transcription of biblical manuscripts are:
        - `hbo-Hebr`, `hbo-Phnx`, `smp-Hebr`, `grc`
- When language changes within a line, the `xml:lang` MUST be applied to the `p` tag directly surrounding the text.

## Text Structure
Text is structured as two levels of nested divs:
- column (represented by a `div` with `type="column"`)
    - line (represented by a `div` with `type="line"`)
Notice that one page is one OS file, so no folio/page-breaks will ever occur inside one file.
For both column and line, the `n` attribute with the correct number SHOULD be given. If it is not given, the number is supplied sequentially, starting from 1
When lines or columns are missing, `n` MUST be applied.

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
- MUST include `type="Verse"`
- MUST include `subtype="{the-versification-scheme}"`
- MUST have `xml:id="A_V_{the-versification-scheme-shorthand}_{the-verse-ID}"`

#### Considerations when adding Verse anchors
Verse anchors are used to align texts in the collation phase. This means that wrong verse anchors will lead to wrong collation and all sorts of Problems.
The strict rule is therefore: When Verse boundaries are uncertain (e.g. words moving between two adjacent verses in different versions),
verse anchors MUST NOT be used.

On the other hand, when different versions have text rearranged or differences in verse numbers and the verse boundaries are certain, it is RECOMMENDED that
verse anchors be supplied in as many versions as possible. This aids collation and indexing of the data.

#### Available Versification schemes
In ALL cases, the abbreviation for books from [OSIS](https://wiki.crosswire.org/OSIS_Book_Abbreviations) MUST be used.
While this may use book abbreviations foreign some(or most) schemes, this simplifies crossreferencing drasticly.

Each manuscript has a natural versification scheme (e.g. `Masoretic` for the Leningradensis).
It MUST be noted (TODO: where??, is this a SHOULD??)

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
- The natural Versification scheme of the present manuscript
When the above schemes do not agree, the `Common` scheme MUST NOT be used.
If other schemes disagree on the verse boundary in question, their anchors MAY be given, but it is NOT RECOMMENDED to do so.
Instead, disagreement for uncommon versification schemes is best noted in a manuscript for which the uncommon scheme is the natural versification scheme.
By way of example: when Aleppo disagrees with BHS, a septuagintal manuscript SHOULD NOT add an anchor for the Aleppo versification.

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
- The entire span of text MUST be enclosed in `<expan>`
- Every character that is physically present and part of the expansion MUST appear in `<abbr>`
- Every character that is physically present but not part of the expansion MUST appear in `<am>`
- Every character that is not physically present but part of the expansion MUST appear in `<ex>`
Thus, the surface form (physically present) is reconstructed as:
- the concatenation of all `<abbr>` and `<am>` elements within the `<expan>`, in order
while the expanded form is reconstructed as:
- the concatenation of all `<abbr>` and `<ex>` elements within the `<expan>`, in order

## Ligatures
TODO: just use gaiji to represent them as glyphs. Use abbreviation as before for NS and the like

## Damaged Characters
This occurs where characters are present, but physically damaged.
- For characters whose existance is obvious but where no physical trace is left, see [Missing](#missing-elements) instead.
- When a reconstruction is to uncertain, use [Illegible Characters](#illegible-characters) instead.

The damaged characters MUST be enclosed in `<damaged>`.
- `@agent` SHOULD be given
- `@cert` SHOULD be given

## Illegible Characters
This occurs when traces of characters are present, but a reconstruction is impossible or considered to uncertain.

The illegible characters MUST be enclosed in `<gap>`.
- `@reason` MUST be `illegible`
- `@unit` MUST be `character`
- When the amount of characters or space missing is known:
    - `@n` MUST be given
- Otherwise
    - `@extent` MUST be given and set to `unknown`
- `@cert` MAY be given and qualifies the certainty in assertaining the amount of missing characters

## Missing Elements
This occurs when the existance of characters, lines, columns etc is obvious (for grammatical-, layout- or other reasons), but no physical trace of those units is left.

The missing characters MUST be enclosed in `<gap>`.
- `@reason` MUST be `lost`
- `@unit` MUST be given and SHOULD be one of `character`, `line`, `column` unless another unit is required.
- When the number of missing units is known:
    - `@n` MUST be given
- Otherwise
    - `@extent` MUST be given and set to `unknown`
- `@cert` MAY be given and qualifies the certainty in assertaining the amount of missing units

## Ancient Corrections
When multiple ancient surface forms are present in a place, these rules apply.

Versions are here considered in (conjectured) temporal order.
- e.g. When a Word was written by Scribe1, struck through by Scribe2 and then another word written atop it by Scribe3, there are three distinct versions.

The entire passage in question MUST be enclosed in `<p>` (this facilitates parsing).

The first version MUST be given without annotations.
- To note the Hand responsible for the first version, `@hand` MAY be used on the enclosing `<p>`.

For each subsequent version, do the following:
- Record the additions relative to the preceding version with `<add>` elements
- Record the deletions relative to the preceding version with `<del>` elements
- Give all of these additions and deletions an `@xml:id` attribute
- Add a `<substJoin>`, with `@target` a space-separated list of the `@xml:id`s given out earlier. (remember the `#`-prefix)
    - You MAY note the hand responsible for this version with `@hand` on the `<substJoin>`.

Critic will calculate the state for each Version.

## Nonstandard Glyphs and Diacritica
### Non-Tiberian Niqud
For vocalization that is not tiberian, you MUST transcribe the vowels with the equivalent tiberian niqud.
An enclosing element (see above, [Source Language](#Defining-the-source-language)) must carry the language tag with the following tags:
- `hbo-Hebr-x-babli-e`
- `hbo-Hebr-x-babli-k`
- `hbo-Hebr-x-palest`

