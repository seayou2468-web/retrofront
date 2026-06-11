#import <Foundation/Foundation.h>
#import <UIKit/UIKit.h>
#import <Flutter/Flutter.h>
#import <UniformTypeIdentifiers/UniformTypeIdentifiers.h>

@interface RetrofrontDocumentPickerDelegate : NSObject <UIDocumentPickerDelegate>
@property(nonatomic, copy) FlutterResult result;
@property(nonatomic, assign) BOOL allowMultiple;
@end

@implementation RetrofrontDocumentPickerDelegate

- (instancetype)initWithResult:(FlutterResult)result allowMultiple:(BOOL)allowMultiple {
  self = [super init];
  if (self) {
    _result = [result copy];
    _allowMultiple = allowMultiple;
  }
  return self;
}

- (void)documentPicker:(UIDocumentPickerViewController *)controller didPickDocumentsAtURLs:(NSArray<NSURL *> *)urls {
  NSMutableArray<NSString *> *paths = [NSMutableArray arrayWithCapacity:urls.count];
  for (NSURL *url in urls) {
    BOOL didStartAccessing = [url startAccessingSecurityScopedResource];
    NSString *path = url.path;
    if (path.length > 0) {
      [paths addObject:path];
    }
    BOOL isDirectory = NO;
    [NSFileManager.defaultManager fileExistsAtPath:path isDirectory:&isDirectory];
    if (didStartAccessing && !isDirectory) {
      [url stopAccessingSecurityScopedResource];
    }
  }
  self.result(paths);
  self.result = nil;
}

- (void)documentPickerWasCancelled:(UIDocumentPickerViewController *)controller {
  self.result(@[]);
  self.result = nil;
}

@end

@interface RetrofrontDocumentPicker : NSObject
@property(nonatomic, strong) FlutterMethodChannel *channel;
@property(nonatomic, strong) RetrofrontDocumentPickerDelegate *delegate;
- (UIViewController *)applicationRootViewController;
@end

@implementation RetrofrontDocumentPicker

+ (void)load {
  [[NSNotificationCenter defaultCenter] addObserver:self selector:@selector(applicationDidFinishLaunching:) name:UIApplicationDidFinishLaunchingNotification object:nil];
}

+ (void)applicationDidFinishLaunching:(NSNotification *)notification {
  dispatch_async(dispatch_get_main_queue(), ^{
    [RetrofrontDocumentPicker.shared installChannelIfNeeded];
  });
}

+ (instancetype)shared {
  static RetrofrontDocumentPicker *shared;
  static dispatch_once_t onceToken;
  dispatch_once(&onceToken, ^{
    shared = [[RetrofrontDocumentPicker alloc] init];
  });
  return shared;
}

- (void)installChannelIfNeeded {
  if (self.channel != nil) {
    return;
  }
  FlutterViewController *controller = (FlutterViewController *)[self applicationRootViewController];
  if (![controller isKindOfClass:FlutterViewController.class]) {
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW, (int64_t)(0.1 * NSEC_PER_SEC)), dispatch_get_main_queue(), ^{
      [self installChannelIfNeeded];
    });
    return;
  }
  self.channel = [FlutterMethodChannel methodChannelWithName:@"retrofront/document_picker" binaryMessenger:controller.binaryMessenger];
  __weak typeof(self) weakSelf = self;
  [self.channel setMethodCallHandler:^(FlutterMethodCall *call, FlutterResult result) {
    [weakSelf handleMethodCall:call result:result];
  }];
}

- (void)handleMethodCall:(FlutterMethodCall *)call result:(FlutterResult)result {
  if ([call.method isEqualToString:@"pickFiles"]) {
    BOOL allowMultiple = [call.arguments[@"allowMultiple"] boolValue];
    [self presentPickerForFolders:NO allowMultiple:allowMultiple result:result];
    return;
  }
  if ([call.method isEqualToString:@"pickDirectory"]) {
    [self presentPickerForFolders:YES allowMultiple:NO result:result];
    return;
  }
  result(FlutterMethodNotImplemented);
}

- (void)presentPickerForFolders:(BOOL)foldersOnly allowMultiple:(BOOL)allowMultiple result:(FlutterResult)result {
  UIViewController *presenter = [self topViewControllerFrom:[self applicationRootViewController]];
  if (presenter == nil) {
    result([FlutterError errorWithCode:@"NO_PRESENTER" message:@"Unable to present the iOS document picker." details:nil]);
    return;
  }

  NSArray<UTType *> *types = foldersOnly ? @[UTTypeFolder] : @[UTTypeItem, UTTypeData, UTTypeArchive];
  UIDocumentPickerViewController *picker = [[UIDocumentPickerViewController alloc] initForOpeningContentTypes:types asCopy:!foldersOnly];
  picker.allowsMultipleSelection = allowMultiple;
  picker.shouldShowFileExtensions = YES;
  self.delegate = [[RetrofrontDocumentPickerDelegate alloc] initWithResult:result allowMultiple:allowMultiple];
  picker.delegate = self.delegate;
  [presenter presentViewController:picker animated:YES completion:nil];
}

- (UIViewController *)applicationRootViewController {
  id<UIApplicationDelegate> appDelegate = UIApplication.sharedApplication.delegate;
  UIWindow *window = nil;
  if ([appDelegate respondsToSelector:@selector(window)]) {
    window = [appDelegate performSelector:@selector(window)];
  }
  if (window == nil) {
    window = UIApplication.sharedApplication.keyWindow;
  }
  if (window == nil) {
    for (UIWindow *candidate in UIApplication.sharedApplication.windows) {
      if (candidate.isKeyWindow) {
        window = candidate;
        break;
      }
    }
  }
  return window.rootViewController;
}

- (UIViewController *)topViewControllerFrom:(UIViewController *)root {
  UIViewController *top = root;
  while (top.presentedViewController != nil) {
    top = top.presentedViewController;
  }
  if ([top isKindOfClass:UINavigationController.class]) {
    return [self topViewControllerFrom:((UINavigationController *)top).visibleViewController];
  }
  if ([top isKindOfClass:UITabBarController.class]) {
    return [self topViewControllerFrom:((UITabBarController *)top).selectedViewController];
  }
  return top;
}

@end
