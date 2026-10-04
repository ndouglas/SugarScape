"""Approved fourteen beats: sampled frame indices, explicitly labeled native periods."""
from camera import Move
from episode import Beat


def beat(name,caption,seconds,shot='teaching',start=0,speed=.001,compare=None):
    seconds+=.4
    return Beat(name,caption,seconds,shot=shot,start_tick=start,ticks_per_second=speed,
        compare=compare,params={'motion_blur':False},overlays=('retirement-population','retirement-panel'),
        camera=stage_camera(name, seconds))

def stage_camera(name, seconds):
    # Different stage views keep the faces large and the feet above captions.
    if name in ('ages', 'renewal', 'habits'):
        eye, end, target, lens = (0,-6.8,3.4), (.25,-6.5,3.2), (0,.7,.45), 43
    elif name == 'decision':
        eye, end, target, lens = (0,-8.6,5.0), (.3,-8.2,4.7), (0,1,.3), 42
    elif name in ('observable', 'meaning'):
        eye, end, target, lens = (-.2,-8.4,4.6), (.25,-8,4.3), (0,1,.35), 41
    elif name in ('groups', 'contact', 'denominator'):
        eye, end, target, lens = (0,-11.5,6.4), (.25,-11,6.1), (0,1.7,.3), 40
    else:
        eye, end, target, lens = (-.35,-9.5,5.1), (.35,-9,4.8), (0,1.4,.3), 43
    return (Move(0,seconds,eye,target,end,target,lens0=lens,lens1=lens),)

BEATS=[
 beat('ages','When do you retire?',7,start=3),
 beat('renewal','Each year they age. When one dies,\na 20-year-old takes its place.',6,speed=.22),
 beat('habits','Some retire as soon as they can.\nSome decide by chance. Most watch their friends.',8,start=1),
 beat('decision','An eligible Flump who imitates retires\nwhen enough eligible friends have retired.',8,start=3),
 beat('quick','With more early retirees,\nretirement spreads quickly.',7,'quick',speed=2.7),
 beat('slow','With fewer, it wavers\nbefore spreading through the crowd.',7,'slow',speed=2.7),
 beat('denominator','Count younger friends too,\nand the cascade changes.',7,'eligible',speed=2.7,compare='all'),
 beat('observable','The most common retirement age\ncan describe only a small retiring minority.',7,'all',start=1),
 beat('groups','Two communities meet. Only one includes\nFlumps who always retire as soon as they can.',7,'groups05',speed=2.7),
 beat('contact','Contact helps one catch up,\nwhile slowing the other.',7,'groups20',speed=2.7,compare='groups05'),
 beat('policy','After 100 years, eligibility\nmoves from 65 to 62.',6,'policy_original',start=31,speed=1.1,compare='policy_revised'),
 beat('thresholds','Raise the imitation thresholds,\nand many runs miss our target within 100 years.',8,'policy_censored',start=34,speed=4.2,compare='policy_original'),
 beat('meaning','How many retire, and at what age,\nare different questions.',7,'all',start=1),
 Beat('end','When to retire - After Axtell & Epstein, 1999; Epstein, 2006\nndouglas.github.io/SugarScape',6,
      shot='teaching',start_tick=3,ticks_per_second=.001,caption_y=.45,
      overlays=('retirement-closing',),params={'motion_blur':False},
      camera=(Move(0,6,(0,-5.5,1.8),(0,0,.6),(0,-5.2,1.7),(0,0,.6),lens0=50,lens1=53),)),
]
