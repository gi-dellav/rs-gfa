
# `J` Jump line (since v1.2)

Jump lines are the mechanism to define the connections of segments which can not be associated with a particular overlap or sequence. Basic usecase is to represent 'gaps' corresponding to unassembled regions, most commonly due to absense or low quality of sequencing data.

`J`-lines specification generally follows one for `L`-lines, using columns 2-4 to specify connected segments and their respective orientations. 
The only difference is that 6th column specifies a signed integer `Distance` (instead of the `Overlap` `CIGAR` string) -- estimated distance between the segments.
The `Distance` can take a `*` value, meaning that the distance is not specified (estimate is unavailable).
Note that the `Distance` can take negative integer values, hinting at an undetected overlap.

Since v1.2 jump connections can be used in the `P`-lines. 
Note that to specify usage of a jump connection rather than a regular link within a path one should use a different separator (`;` instead of `,`). For details and examples see "Extension to use jump connections" subsection the `P`-line description.

`J`-lines can also be used to specify _shortcut_ connections that do not correspond to any missing overlap or absent sequence.
Shortcuts are primarily intended to be used within the `P`-lines to define arbitrary assembly scaffolds.
Shortcut `J`-lines must be marked with a special tag: `SC:i:1`.

## Required fields

| Column | Field        | Type      | Regexp                   | Description
|--------|--------------|-----------|--------------------------|------------------
| 1      | `RecordType` | Character | `J`                      | Record type
| 2      | `From`       | String    | `[!-)+-<>-~][!-~]*`      | Name of segment
| 3      | `FromOrient` | String    | `+\|-`                   | Orientation of From segment
| 4      | `To`         | String    | `[!-)+-<>-~][!-~]*`      | Name of segment
| 5      | `ToOrient`   | String    | `+\|-`                   | Orientation of `To` segment
| 6      | `Distance`   | String    | `\*\|[-+]?[0-9]+`        | Optional estimated distance between the segments

## Optional fields

| Tag  | Type | Description
|------|------|------------
| `SC` | `i`  | 1 indicates indirect shortcut connections. Only 0/1 allowed.

## Example

The following lines describe the jump between reverse complement of segment 1 and segment 2, with estimated distance of 100 and the  'shortcut' between segment 2 and reverse complement of segment 3 with unspecified distance.
```
J  1 - 2 + 100
J  2 + 3 - * SC:i:1
```
