"""Following the Crowd, episode 7: approved thresholds storyboard."""
from camera import Move
from episode import Beat

S={'colors':'participation','motion_blur':False}
A=('thresholds-panel','thresholds-markers')


def beat(name,caption,shot,seconds=7,speed=1,compare=None,close=False):
    network=shot in ('sparse','middle','dense','dense-large')
    eye,at=((10,-60,70),(10,2,0)) if network else ((4,-18,21),(4,0,0))
    if close: eye,at=(-2,0,5),(-2,4.5,.3)
    # Keep all three teaching labels in view throughout the close-up move.
    lens=30 if close else 34
    extra=('thresholds-comparison',) if compare else ()
    if name=='network': extra+=('thresholds-neighborhood',)
    return Beat(name,caption,seconds,shot=shot,ticks_per_second=speed,params=S,
                overlays=A+extra+(('thresholds-links',) if network or shot in ('friends','rescue') else ()),
                compare=compare,start_tick=500 if name=='ceilings' else 0,camera=(Move(0,seconds,eye,at,tuple(x+y for x,y in zip(eye,(0,1,-.4))),at,lens0=lens,lens1=lens),))


BEATS=[
    beat('crowd','100 Flumps, each waiting for enough others to join.','uniform',6,.01),
    beat('instigator','One needs nobody. Another needs one.\nThe next needs two.','uniform',7,.01,close=True),
    beat('chain','One starts. Each new arrival brings in the next.\nAll 100 join.','uniform',8,16),
    beat('change','Raise just one threshold from 1 to 2.','perturbed',6,.01,close=True),
    beat('stalled','Now only the instigator joins.\nAlmost the same crowd. A very different outcome.','perturbed',7,1,compare='uniform'),
    beat('city','Granovetter imagined random crowds from a uniform city.\nHe calculated that about half would stop at zero or one.','city',8,16),
    beat('friends','Count friends twice, and the chain usually stops early.','friends',7,2),
    beat('rescue','In the stalled crowd, stronger friends\ncan bring a few more along.','rescue',7,2),
    beat('ceilings','Let some leave when the crowd gets too big,\nand participation can rise and fall.','ceilings',8,13.5),
    beat('network','Watts put the rule on a network.\nEach Flump watches only its neighbors.','middle',7,.01),
    beat('sparse','With few links, most sparks stay small.','sparse',7,4),
    beat('middle','With a middling number of links,\nmost sparks spread through nearly the whole network.','middle',8,5),
    beat('dense','In our 1,000-node networks, many sparks die out.\nSome still sweep nearly the whole network.','dense',9,5,compare='dense-large'),
    Beat('end','The riot that needs one person - After Granovetter, 1978; Watts, 2002\nndouglas.github.io/SugarScape',5,
         caption_y=.45,camera=(Move(0,5,(0,-5.5,1.8),(0,0,.6),(0,-5.2,1.7),(0,0,.6),lens0=50,lens1=53),)),
]
