#
#  TutorialLevels.py -- the example level set from docs/creating-levels.md
#
#  Play it with:
#      python3 bubbob/bb.py docs/examples/TutorialLevels.py
#  or copy it into bubbob/levels/ so that it shows up in the web launcher.
#

import boarddef
from boarddef import LNasty, LMonky, LGhosty, LFlappy
from boarddef import LSpringy, LOrcy, LGramy, LBlitzy
from boarddef import RNasty, RMonky, RGhosty, RFlappy
from boarddef import RSpringy, ROrcy, RGramy, RBlitzy


# Level 1: the smallest useful level.  Walls ('#'), four monsters marked
# with the letters 'a' and 'b', and the default winds.
class level01(boarddef.Level):
    a = LNasty      # 'a' cells become a Nasty that starts walking right
    b = RNasty      # 'b' cells become a Nasty that starts walking left

    walls = """
#############################
##                         ##
##                         ##
##                         ##
##                         ##
##                         ##
##                         ##
##                         ##
##       a         b       ##
#######   #########   #######
##                         ##
##                         ##
##                         ##
##       a         b       ##
#######   #########   #######
##                         ##
##                         ##
##                         ##
##                         ##
##                         ##
##                         ##
##                         ##
##                         ##
#############################
"""


# Level 2: hand-made winds and a special-bubble flag.  Bubbles drift
# along the wind map: they rise in the two side corridors, are pushed
# towards the middle at the top, and gather in the 'x' cells.
class level02(boarddef.Level):
    m = RMonky                # Monkies throw things at you
    g = LGhosty, RGhosty      # one letter can stand for several monsters
    letter = 1                # E-X-T-E-N-D letter bubbles may appear

    walls = """
#############################
##                         ##
##                         ##
##                         ##
##     g                   ##
##                         ##
##                         ##
#####   #############   #####
##                         ##
##                         ##
##                         ##
##           m             ##
##      ###########        ##
##                         ##
##                         ##
##                         ##
#####   #############   #####
##                         ##
##                         ##
##                         ##
##                         ##
##                         ##
##                         ##
#############################
"""

    winds = """
>>                         <<
>>>>>>>>>>>>xxxxx<<<<<<<<<<<<
>>>>>>>>>>>>xxxxx<<<<<<<<<<<<
>>^^^^^^^^^^^^^^^^^^^^^^^^^<<
>>^^^^^^^^^^^^^^^^^^^^^^^^^<<
>>^^^^^^^^^^^^^^^^^^^^^^^^^<<
>>^^^^^^^^^^^^^^^^^^^^^^^^^<<
>>^^^^^^^^^^^^^^^^^^^^^^^^^<<
>>^^^^^^^^^^^^^^^^^^^^^^^^^<<
>>^^^^^^^^^^^^^^^^^^^^^^^^^<<
>>^^^^^^^^^^^^^^^^^^^^^^^^^<<
>>^^^^^^^^^^^^^^^^^^^^^^^^^<<
>>^^^^^^^^^^^^^^^^^^^^^^^^^<<
>>^^^^^^^^^^^^^^^^^^^^^^^^^<<
>>^^^^^^^^^^^^^^^^^^^^^^^^^<<
>>^^^^^^^^^^^^^^^^^^^^^^^^^<<
>>^^^^^^^^^^^^^^^^^^^^^^^^^<<
>>^^^^^^^^^^^^^^^^^^^^^^^^^<<
>>^^^^^^^^^^^^^^^^^^^^^^^^^<<
>>^^^^^^^^^^^^^^^^^^^^^^^^^<<
>>^^^^^^^^^^^^^^^^^^^^^^^^^<<
>>^^^^^^^^^^^^^^^^^^^^^^^^^<<
>>^^^^^^^^^^^^^^^^^^^^^^^^^<<
>>                         <<
"""


# Level 3: holes in the floor and ceiling (things that fall out of the
# bottom come back in at the top), special bubbles coming from the top,
# and a mix of flying and jumping monsters.
class level03(boarddef.Level):
    f = LFlappy
    s = RSpringy
    o = LOrcy, ROrcy
    fire = water = 1          # fire and water bubbles may appear
    top = 1                   # special bubbles enter from the top

    walls = """
###########       ###########
##                         ##
##                         ##
##   f                 f   ##
##                         ##
##                         ##
###########       ###########
##                         ##
##                         ##
##             o           ##
##        #########        ##
##                         ##
##                         ##
####                     ####
##                         ##
##      s           s      ##
##    ####         ####    ##
##                         ##
##                         ##
##                         ##
##                         ##
##                         ##
##                         ##
###########       ###########
"""


# The last level of a set has no monsters: it is the "bonus" level
# that is shown when the game is over.  Its name must sort last.
class levelFinal(boarddef.Level):

    walls = """
#############################
##                         ##
##                         ##
##                         ##
##                         ##
##                         ##
##                         ##
##        #########        ##
##                         ##
##                         ##
##                         ##
##                         ##
##     ###############     ##
##                         ##
##                         ##
##                         ##
##                         ##
##  #####################  ##
##                         ##
##                         ##
##                         ##
##                         ##
##                         ##
#############################
"""
