//
//  CitraCore.h
//  Citra
//
//  Public API for Citra.framework.
//

#import <Foundation/Foundation.h>
#import <QuartzCore/CAMetalLayer.h>
#import <UIKit/UIKit.h>

NS_ASSUME_NONNULL_BEGIN

typedef NS_ENUM(NSUInteger, CitraVirtualControllerAnalogType) {
    CitraVirtualControllerAnalogTypeCirclePad = 713,
    CitraVirtualControllerAnalogTypeCirclePadUp = 714,
    CitraVirtualControllerAnalogTypeCirclePadDown = 715,
    CitraVirtualControllerAnalogTypeCirclePadLeft = 716,
    CitraVirtualControllerAnalogTypeCirclePadRight = 717,
    CitraVirtualControllerAnalogTypeCStick = 718,
    CitraVirtualControllerAnalogTypeCStickUp = 719,
    CitraVirtualControllerAnalogTypeCStickDown = 720,
    CitraVirtualControllerAnalogTypeCStickLeft = 771,
    CitraVirtualControllerAnalogTypeCStickRight = 772
};

typedef NS_ENUM(NSUInteger, CitraVirtualControllerButtonType) {
    CitraVirtualControllerButtonTypeA = 700,
    CitraVirtualControllerButtonTypeB = 701,
    CitraVirtualControllerButtonTypeX = 702,
    CitraVirtualControllerButtonTypeY = 703,
    CitraVirtualControllerButtonTypeStart = 704,
    CitraVirtualControllerButtonTypeSelect = 705,
    CitraVirtualControllerButtonTypeHome = 706,
    CitraVirtualControllerButtonTypeTriggerZL = 707,
    CitraVirtualControllerButtonTypeTriggerZR = 708,
    CitraVirtualControllerButtonTypeDirectionalPadUp = 709,
    CitraVirtualControllerButtonTypeDirectionalPadDown = 710,
    CitraVirtualControllerButtonTypeDirectionalPadLeft = 711,
    CitraVirtualControllerButtonTypeDirectionalPadRight = 712,
    CitraVirtualControllerButtonTypeTriggerL = 773,
    CitraVirtualControllerButtonTypeTriggerR = 774,
    CitraVirtualControllerButtonTypeDebug = 781,
    CitraVirtualControllerButtonTypeGPIO14 = 782
};

typedef NS_ENUM(NSUInteger, CitraImportResultStatus) {
    CitraImportResultStatusSuccess,
    CitraImportResultStatusErrorFailedToOpenFile,
    CitraImportResultStatusErrorFileNotFound,
    CitraImportResultStatusErrorAborted,
    CitraImportResultStatusErrorInvalid,
    CitraImportResultStatusErrorEncrypted,
};

typedef NS_ENUM(NSUInteger, CitraKeyboardButtonConfig) {
    CitraKeyboardButtonConfigSingle,
    CitraKeyboardButtonConfigDual,
    CitraKeyboardButtonConfigTriple,
    CitraKeyboardButtonConfigNone
};

typedef NS_ENUM(uint8_t, CitraKernelMemoryMode) {
    CitraKernelMemoryModeProd = 0,
    CitraKernelMemoryModeDev1 = 2,
    CitraKernelMemoryModeDev2 = 3,
    CitraKernelMemoryModeDev3 = 4,
    CitraKernelMemoryModeDev4 = 5
};

typedef NS_ENUM(uint8_t, CitraNew3DSKernelMemoryMode) {
    CitraNew3DSKernelMemoryModeLegacy = 0,
    CitraNew3DSKernelMemoryModeProd = 1,
    CitraNew3DSKernelMemoryModeDev1 = 2,
    CitraNew3DSKernelMemoryModeDev2 = 3
};

@interface CitraCoreVersion : NSObject
@property (nonatomic) uint32_t major, minor, revision;
- (instancetype)initWithCoreVersion:(uint32_t)coreVersion;
@end

@interface CitraGameInformation : NSObject
@property (nonatomic, strong) CitraCoreVersion *coreVersion;
@property (nonatomic) uint64_t identifier;
@property (nonatomic) CitraKernelMemoryMode kernelMemoryMode;
@property (nonatomic) CitraNew3DSKernelMemoryMode new3DSKernelMemoryMode;
@property (nonatomic, strong) NSString *regions, *publisher, *title;
@property (nonatomic, strong, nullable) NSData *icon;
- (nullable instancetype)initWithURL:(NSURL *)url;
@end

@interface CitraSaveStateInfo : NSObject
@property (nonatomic) uint32_t slot;
@property (nonatomic) uint64_t time;
@property (nonatomic, strong) NSString *buildName;
@property (nonatomic) int status;
- (instancetype)initWithSlot:(uint32_t)slot
                        time:(uint64_t)time
                   buildName:(NSString *)buildName
                      status:(int)status;
@end

@interface CitraKeyboardConfig : NSObject
@property (nonatomic, strong, nullable) NSString *hintText;
@property (nonatomic, assign) CitraKeyboardButtonConfig buttonConfig;
@property (nonatomic, assign) uint16_t maxTextSize;
- (instancetype)initWithHintText:(NSString *_Nullable)hintText
                    buttonConfig:(CitraKeyboardButtonConfig)buttonConfig
                     maxTextSize:(uint16_t)maxTextSize;
@end

@interface CitraCIAInfo : NSObject
@property (nonatomic) uint64_t identifier;
@property (nonatomic, copy, nullable) NSString *contentPath;
@property (nonatomic, copy, nullable) NSString *titlePath;
@end

@interface CitraSaveStateResult : NSObject
@property (nonatomic) BOOL isSuccess;
@property (nonatomic, copy) NSString *path;
@end

@interface CitraCore : NSObject

+ (instancetype)shared NS_SWIFT_NAME(shared());

@property (class, nonatomic, strong, nullable) CitraGameInformation *currentGameInfo;
@property (class, nonatomic, copy, nullable) void (^openKeyboardAction)(NSString *_Nullable hintText,
                                                                        CitraKeyboardButtonConfig keyboardType,
                                                                        uint16_t maxTextSize);

- (nullable CitraGameInformation *)informationForGameAtURL:(NSURL *)url NS_SWIFT_NAME(information(for:));

- (void)allocateVulkanLibrary;
- (void)deallocateVulkanLibrary;

- (void)allocateMetalLayer:(CAMetalLayer *)layer
                    withSize:(CGSize)size
                 isSecondary:(BOOL)isSecondary NS_SWIFT_NAME(allocateMetalLayer(for:with:isSecondary:));
- (void)deallocateMetalLayers;

- (void)insertCartridgeAndBootWithURL:(NSURL *)url
                         advancedMode:(BOOL)advancedMode
                           jitSupport:(BOOL)jitSupport NS_SWIFT_NAME(insertCartridgeAndBoot(with:advancedMode:jitSupport:));

- (void)assignGameInfoWithURL:(nullable NSURL *)url NS_SWIFT_NAME(assignGameInfo(with:));

- (CitraImportResultStatus)importGameAtURL:(NSURL *)url NS_SWIFT_NAME(importGame(at:));

- (void)touchBeganAtPoint:(CGPoint)point NS_SWIFT_NAME(touchBegan(at:));
- (void)touchEnded;
- (void)touchMovedAtPoint:(CGPoint)point NS_SWIFT_NAME(touchMoved(at:));

- (void)virtualControllerButtonDown:(CitraVirtualControllerButtonType)button;
- (void)virtualControllerButtonUp:(CitraVirtualControllerButtonType)button;
- (void)thumbstickMoved:(CitraVirtualControllerAnalogType)analog x:(float)x y:(float)y;

- (BOOL)isPaused;
- (void)pausePlay:(BOOL)pausePlay;
- (void)stop;
- (void)reset;
- (BOOL)running;
- (BOOL)stopped;

- (void)orientationChangeWithOrientation:(UIInterfaceOrientation)orientation
                               metalView:(UIView *)metalView NS_SWIFT_NAME(orientationChange(with:using:));

- (CitraCIAInfo *)getCIAInfoWithURL:(NSURL *)url isSdmc:(BOOL)isSdmc NS_SWIFT_NAME(getCIAInfo(url:isSdmc:));

- (NSArray<NSURL *> *)installed;
- (NSArray<NSURL *> *)system;

- (void)updateSettingsWithAdvancedMode:(BOOL)advancedMode NS_SWIFT_NAME(updateSettings(advancedMode:));

@property (nonatomic) uint16_t stepsPerHour;
@property (nonatomic, readonly) NSInteger saveStateCount;

- (BOOL)loadState NS_SWIFT_NAME(loadState());
- (BOOL)loadStateWithSlot:(uint32_t)slot NS_SWIFT_NAME(loadState(_:));

- (CitraSaveStateResult *)saveState NS_SWIFT_NAME(saveState());

- (NSArray<CitraSaveStateInfo *> *)savesForIdentifier:(uint64_t)identifier NS_SWIFT_NAME(saves(for:));
- (NSString *)saveStatePathForIdentifier:(uint64_t)identifier slot:(uint32_t)slot NS_SWIFT_NAME(saveStatePath(for:slot:));
- (nullable NSString *)saveStatePathForRunningGameWithSlot:(uint32_t)slot NS_SWIFT_NAME(saveStatePathForRunningGame(slot:));

- (BOOL)loadAmiiboWithPath:(NSString *)path NS_SWIFT_NAME(loadAmiibo(path:));
- (BOOL)isSearchingAmiibo;

- (void)jumpToHome;
- (nullable NSString *)getTitlePathWithIdentifier:(uint64_t)identifier isSdmc:(BOOL)isSdmc NS_SWIFT_NAME(getTitlePath(identifier:isSdmc:));
- (nullable NSString *)getCIAContentPathWithIdentifier:(uint64_t)identifier isSdmc:(BOOL)isSdmc NS_SWIFT_NAME(getCIAContentPath(identifier:isSdmc:));

- (void)setSimBlowing:(BOOL)start NS_SWIFT_NAME(setSimBlowing(start:));
- (void)setFrameLimit:(uint16_t)limit NS_SWIFT_NAME(setFrameLimit(_:));

@end

NS_ASSUME_NONNULL_END
