#import "RetrofrontIosBridgePlugin.h"
#import <Foundation/Foundation.h>
#import <UIKit/UIKit.h>
#import <UniformTypeIdentifiers/UniformTypeIdentifiers.h>

@interface RetrofrontDocumentPickerDelegate : NSObject <UIDocumentPickerDelegate>
@property(nonatomic, copy) FlutterResult result;
@property(nonatomic, assign) BOOL foldersOnly;
@end

@implementation RetrofrontDocumentPickerDelegate

- (instancetype)initWithResult:(FlutterResult)result foldersOnly:(BOOL)foldersOnly {
  self = [super init];
  if (self) {
    _result = [result copy];
    _foldersOnly = foldersOnly;
  }
  return self;
}

- (void)documentPicker:(UIDocumentPickerViewController *)controller didPickDocumentsAtURLs:(NSArray<NSURL *> *)urls {
  NSMutableArray<NSString *> *paths = [NSMutableArray arrayWithCapacity:urls.count];
  NSFileManager *fileManager = NSFileManager.defaultManager;
  NSURL *importsDirectory = [NSURL fileURLWithPath:[NSTemporaryDirectory() stringByAppendingPathComponent:@"RetrofrontImports"] isDirectory:YES];
  [fileManager createDirectoryAtURL:importsDirectory withIntermediateDirectories:YES attributes:nil error:nil];

  for (NSURL *url in urls) {
    BOOL didStartAccessing = [url startAccessingSecurityScopedResource];
    NSString *path = url.path;
    BOOL isDirectory = NO;
    [fileManager fileExistsAtPath:path isDirectory:&isDirectory];

    if (path.length > 0 && !self.foldersOnly && !isDirectory) {
      NSString *uniqueName = [NSString stringWithFormat:@"%@-%@", NSUUID.UUID.UUIDString, url.lastPathComponent ?: path.lastPathComponent];
      NSURL *destination = [importsDirectory URLByAppendingPathComponent:uniqueName isDirectory:NO];
      [fileManager removeItemAtURL:destination error:nil];
      if ([fileManager copyItemAtURL:url toURL:destination error:nil]) {
        path = destination.path;
      }
    }

    if (path.length > 0) {
      [paths addObject:path];
    }

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

@interface RetrofrontIosBridgePlugin ()
@property(nonatomic, strong) RetrofrontDocumentPickerDelegate *delegate;
@end

@implementation RetrofrontIosBridgePlugin

+ (void)registerWithRegistrar:(NSObject<FlutterPluginRegistrar> *)registrar {
  FlutterMethodChannel *channel = [FlutterMethodChannel methodChannelWithName:@"retrofront/document_picker" binaryMessenger:registrar.messenger];
  RetrofrontIosBridgePlugin *instance = [[RetrofrontIosBridgePlugin alloc] init];
  [registrar addMethodCallDelegate:instance channel:channel];
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
  self.delegate = [[RetrofrontDocumentPickerDelegate alloc] initWithResult:result foldersOnly:foldersOnly];
  picker.delegate = self.delegate;
  [presenter presentViewController:picker animated:YES completion:nil];
}


- (UIViewController *)applicationRootViewController {
  UIWindow *window = nil;
  id<UIApplicationDelegate> appDelegate = UIApplication.sharedApplication.delegate;
  if ([appDelegate respondsToSelector:@selector(window)]) {
    window = [appDelegate performSelector:@selector(window)];
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
