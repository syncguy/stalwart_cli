# NT 6.1 experimental patch

Source: Ratatui ratatui-crossterm 0.1.2, exact tag ratatui-crossterm-v0.1.2, commit e665c36cb14752a61cd777fbd06dbef8474f2add. MIT license retained. Original source blob 07ba17e02c45cf3f3dbc919ff46e8029e686f679. Original registry checksum 567584a3b0e6a8203c23de40b4861497266725eb5363dbfd18a1edd603cca9f0. Manifest workspace inheritance expanded to standalone values from the same tag, workspace lints omitted. No dependency version change.

Backend draw now gates SetUnderlineColor (actual color and Reset) on the same Windows ANSI support decision as Crossterm queue!. Legacy WinAPI does not implement that command. All other operations and error propagation remain unchanged; modern ANSI terminals retain underline colors. CI checks patch selection and records SHA-256. Retire once upstream supports capability handling.
