"""Ants at two food piles: the fourteen approved beats."""
from camera import Move
from episode import Beat


def beat(name,caption,shot,seconds=7,start=0,speed=1):
    big=shot in ('large','ring','random','independent')
    scale=2.85 if big else 1
    eye=(10*scale,-34*scale,43*scale); at=(10*scale,0,0)
    return Beat(name,caption,seconds,shot=shot,start_tick=start,ticks_per_second=speed,
                params={'colors':'ants','motion_blur':False},overlays=('ants-stage','ants-panel'),
                camera=(Move(0,seconds,eye,at,eye,at,lens0=34,lens1=34),))

BEATS=[
    beat('piles','Two identical food piles.\nReal ants sometimes crowded one, about 80–20.','strong',7,0,0.01),
    beat('choices',"Kirman's Flumps choose between two sources.\nNeither is better.",'strong',7,0,1),
    beat('meet','Meet another Flump, and you may copy its choice.','micro-recruit',7,2,0.25),
    beat('self','Occasionally, a Flump switches on its own.','micro-self',6,212,0.3),
    beat('crowd','With strong recruiting,\nnearly everyone crowds one source.','strong',8,4,1),
    beat('flip','Then the crowd can flip,\nwithout the food changing.','strong',9.4,17,3),
    beat('average',"Both sides get turns.\nA short run needn't average half and half.",'short',8,0,30),
    beat('splits','The long-run peaks are at all-or-nothing.\n80–20 is not a preferred split.','strong',7,17,2),
    beat('pull','Kirman suggested stronger attraction to the majority.\nOur version favors splits near 18–82.','pull',7,0,32),
    beat('more','Ten times the Flumps, with the same habits:\nless time crowded at one source.','large',8,0,32),
    beat('neighbors','Alfarano and Milaković counted neighbors,\nrather than one partner per meeting.','ring',8,0,0.01),
    beat('growth','With their rule, growth weakens herding on rings.\nRandom networks keep their swings.','random',8,0,15),
    beat('independent','A few Flumps who never copy\ncan calm the crowd.','independent',8,0,15),
    Beat('end','Ants at two food piles - After Kirman, 1993; Alfarano & Milaković, 2007\nndouglas.github.io/SugarScape',5,caption_y=.45,camera=(Move(0,5,(0,-5.5,1.8),(0,0,.6),(0,-5.2,1.7),(0,0,.6),lens0=50,lens1=53),)),
]
