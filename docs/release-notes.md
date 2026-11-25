# Release notes

Write a short English summary. GitHub's prerelease flag controls who receives the update. The default update dialog offers the new version and asks before downloading.

## When support is ending

Clients starting with 0.9.14 recognize this optional HTML comment on the first nonblank line of the selected release body:

~~~markdown
<!-- garlic-support-floor: 0.9.15 -->
Update before support for older clients ends.
~~~

The marker in this example warns installed versions below 0.9.15. Version 0.9.15 itself gets the ordinary prompt. Choose a floor at or below the version offered in the release. Omit the marker for routine releases. The warning can be dismissed; it neither disables the client nor starts a download. Older clients keep their existing update prompt.

## Required updates

Starting with 0.9.15, place `<!-- garlic-required-floor: 0.9.16 -->` on the first nonblank line to require an update for older installations. Choose a floor no higher than the offered version. Clients apply the marker after selecting their release channel. Installation requires confirmation. Windows offers to restart Garlic after installation.
