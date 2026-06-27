// Fails (exit 1) if any file under a `domain/` directory imports Flutter, dio,
// or dart:io — the client analogue of the backend's ci/check-boundaries.sh.
import 'dart:io';

void main() {
  final forbidden = [
    RegExp(r'''import\s+['"]package:flutter/'''),
    RegExp(r'''import\s+['"]package:dio/'''),
    RegExp(r'''import\s+['"]dart:io'''),
  ];
  final sep = Platform.pathSeparator;
  final violations = <String>[];

  for (final entity in Directory('lib').listSync(recursive: true)) {
    if (entity is! File || !entity.path.endsWith('.dart')) continue;
    if (!entity.path.contains('${sep}domain$sep')) continue;
    final src = entity.readAsStringSync();
    for (final re in forbidden) {
      if (re.hasMatch(src)) {
        violations.add('${entity.path}: forbidden import (${re.pattern})');
      }
    }
  }

  if (violations.isNotEmpty) {
    stderr.writeln('Domain boundary violations:');
    for (final v in violations) {
      stderr.writeln('  $v');
    }
    exit(1);
  }
  stdout.writeln('domain boundary OK');
}
