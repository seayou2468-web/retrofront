//
//  CheatsManager.h
//  Cytrus
//
//  Created by Jarrod Norwell on 19/10/2024.
//  Copyright © 2024 Jarrod Norwell. All rights reserved.
//

#import <Foundation/Foundation.h>

#ifdef __cplusplus
#include <algorithm>
#include <memory>
#include <utility>

#include "core/cheats/cheats.h"
#endif

NS_ASSUME_NONNULL_BEGIN

NS_SWIFT_SENDABLE
@interface CitraCheat : NSObject
@property (nonatomic, assign) BOOL enabled;
@property (nonatomic, strong) NSString *name, *code, *comments;

-(CitraCheat *) initWithEnabled:(BOOL)enabled name:(NSString *)name code:(NSString *)code comments:(NSString *)comments;
@end

@interface CitraCheatsManager : NSObject {
    uint64_t _identifier;
}

-(CitraCheatsManager *) initWithIdentifier:(uint64_t)identifier;

-(void) loadCheats;
-(void) saveCheats;

-(NSArray<CitraCheat *> *) getCheats;

-(void) removeCheatAtIndex:(NSInteger)index;
-(void) toggleCheat:(CitraCheat *)cheat;
-(void) updateCheat:(CitraCheat *)cheat atIndex:(NSInteger)index;
//Manic修改 获取作弊文件的路径
-(NSString *) cheatFilePath;
@end

NS_ASSUME_NONNULL_END
