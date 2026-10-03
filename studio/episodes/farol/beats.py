"""The fourteen approved captions; frame indices deliberately differ from rounds."""
from camera import Move
from episode import Beat


def beat(name,caption,shot,seconds=7,start=400,speed=2,compare=None):
    seconds += .4  # Restore the 12-frame dissolve overlap in the final cut.
    eye=(10,-34,43); at=(10,0,0)
    return Beat(name,caption,seconds,shot=shot,start_tick=start,ticks_per_second=speed,
                compare=compare,params={'colors':'farol','motion_blur':False},
                overlays=('farol-stage','farol-panel'),
                camera=(Move(0,seconds,eye,at,eye,at,lens0=34,lens1=34),))


BEATS=[
    beat('bar','100 Flumps consider a night out.\nThe bar is crowded at 60.','accuracy',6,0,.001),
    beat('forecasts','Each has a few forecasts,\nbuilt from past attendance.','accuracy',7,401,.001),
    beat('decide','Go if the best forecast says fewer than 60.\nOtherwise, stay home.','accuracy',7,400,.16),
    beat('react','A quiet week can draw a crowd.\nA crowd can send many home.','accuracy',6,400,1),
    beat('mean','Attendance averages about 60.\nBut the swings are wide.','accuracy',7,400,4),
    beat('coin','A 60%-go coin also averages 60,\nwith much smaller swings.','random',7,400,4,'accuracy'),
    beat('advice','Rate forecasts by whether their advice was right,\nand the swings shrink.','advice',7,400,4,'accuracy'),
    beat('shared','Give everyone the same forecast bank,\nand they all go—or all stay home.','shared',7,400,1),
    beat('minority','Now there are two choices.\nThe smaller crowd wins.','m6',6,400,1),
    beat('memory','Each follows its best-scoring strategy,\nusing the recent winning sides.','teaching',8,2000,.14),
    beat('short','With short memories,\nthey crowd the same side.','m2',7,400,4),
    beat('middle','An intermediate memory helps them split\nalmost evenly, without talking.','m6',7,400,4),
    beat('long','Too much memory, and coordination\nfalls back near chance.','m12',7,400,4),
    Beat('end',"Nobody goes, it's too crowded - After Arthur, 1994; Challet & Zhang, 1997\nndouglas.github.io/SugarScape",6,
         caption_y=.45,camera=(Move(0,6,(0,-5.5,1.8),(0,0,.6),(0,-5.2,1.7),(0,0,.6),lens0=50,lens1=53),)),
]
