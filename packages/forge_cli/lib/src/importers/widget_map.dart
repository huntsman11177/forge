/// Mapping helpers between Flutter widget identifiers and Forge canonical names.
///
/// This can be expanded as we support additional frameworks. For now we keep a
/// small curated list to normalize common Flutter widgets.
class WidgetCanonicalizer {
  static const Map<String, String> _canonicalNames = {
    'Scaffold': 'Scaffold',
    'AppBar': 'AppBar',
    'MaterialApp': 'MaterialApp',
    'Text': 'Text',
    'Column': 'Column',
    'Row': 'Row',
    'Container': 'Container',
    'Center': 'Center',
    'Padding': 'Padding',
    'SizedBox': 'SizedBox',
    'ListView': 'ListView',
    'ElevatedButton': 'ElevatedButton',
    'Icon': 'Icon',
  };

  /// Returns the canonical Forge widget name for a given [identifier]. If the
  /// widget is unknown, the original identifier is returned.
  static String canonicalize(String identifier) {
    return _canonicalNames[identifier] ?? identifier;
  }
}
