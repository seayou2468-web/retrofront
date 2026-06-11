Pod::Spec.new do |s|
  s.name             = 'retrofront_ios_bridge'
  s.version          = '0.1.0'
  s.summary          = 'Objective-C iOS bridge for Retrofront.'
  s.description      = 'Registers Retrofront iOS method channels without adding Swift runtime dependencies.'
  s.homepage         = 'https://example.invalid/retrofront'
  s.license          = { :type => 'MIT' }
  s.author           = { 'Retrofront' => 'retrofront@example.invalid' }
  s.source           = { :path => '.' }
  s.source_files     = 'Classes/**/*'
  s.dependency 'Flutter'
  s.platform = :ios, '14.0'
  s.requires_arc = true
  s.pod_target_xcconfig = {
    'DEFINES_MODULE' => 'NO',
    'SWIFT_VERSION' => '',
    'ALWAYS_EMBED_SWIFT_STANDARD_LIBRARIES' => 'NO'
  }
end
