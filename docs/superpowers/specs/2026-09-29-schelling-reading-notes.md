# Schelling's segregation models: what he specified, claimed, and left open

Research notes for the "Schelling's baby" episode(s). Sources are the PDFs in
`papers/schelling/`. Page numbers for Schelling 1971 are journal pages (J. Math. Sociol. 1:143–186).
PDF page n is journal page 142+n. I read every 1971 page as an image. I also transcribed the
key figures and re-ran the 1-D worked example and the 2-D starting board in a small Python
check (scratchpad `sim/`, described in section 5).

Legend for claims: **(a)** a general or logical claim; **(b)** a result from a worked example or
figure, with his numbers; **(c)** a conjecture, a "suggests", or a hypothesis he says his samples
can't support.

---------------------------------------------------------------------------------------------------

## 0. Paper map (1971)

| pp. | Section | Model |
|---|---|---|
| 143–148 | Intro, "Some Quantitative Constraints", "Separating Mechanisms" | logical arithmetic of ratios |
| 149–154 | SPATIAL PROXIMITY MODEL: Linear Distribution, Variations, Restricted Movement | **1-D line** |
| 154–166 | Area Distribution, Intensity, Unequal Demands, Unequal Numbers, Population Densities, Size of Neighborhood, Congregationist, Integrationist | **2-D checkerboard** |
| 167–181 | BOUNDED-NEIGHBORHOOD MODEL: straight-line tolerance, translation, static viability, dynamics, alternative schedules and numbers, limiting numbers, varying tolerance, varieties (results 1–7), integrationist, policies | **compartment / tolerance schedules** |
| 181–186 | TIPPING | bounded-neighborhood applied to a neighborhood with fixed capacity |

Footnote on p.143: the study was issued as RAND RM-6014-RC in May 1969. 1969 AER P&P (pp. 488–493) is a
short version containing only the 1-D line and the bounded-neighborhood model (no 2-D board).

---------------------------------------------------------------------------------------------------

## 1. Logical constraints (pp.147–148; 1969 pp.488–489)

- (a) "within a given set of boundaries, not both groups ... can enjoy numerical superiority ... if each insists on being a *local* majority, there is only one mixture that will satisfy them—complete segregation." (p.147)
- (a) Whites ≥3/4 and blacks ≥1/3: "it won't work". Whites ≥2/3 and blacks ≥1/5: "a small range of mixtures" (p.147).
- (a) Small-number constraints: a neighbor on either side means the minimum nonzero opposite share is 50%. Hospital: if blacks hold 1/6 of beds, 4 per room, then "at least 40% of the whites will be in all-white rooms". A college that wants 10% black students must admit 40% black freshmen (p.147).
- (a) If both colors tolerate being at most a 25% minority, "initial mixtures ranging from 25% to 75% will survive but initial mixtures more extreme than that will lose their minority members" (p.148). With lower limits on minority status and initial complete segregation, "Complete segregation is then a stable equilibrium ... somebody has to move first and nobody will." (p.148)

---------------------------------------------------------------------------------------------------

## 2. The 1-D line model ("Linear Distribution", pp.149–154; 1969 pp.489–491)

### 2.1 Rules as specified

- **Population and initial arrangement.** Stars (+) and zeros (0) "correspond to the odd and even digits in a column of random numbers. It turns out that there are 35 stars and 35 zeros" (p.149). So the order is random, from a random-number table, and the counts were not forced to be equal. Fig.1 has 70 people. A second example (Fig.3) has 72 people from "another column in the same table" (p.151). He also notes that "the 70 people who fit within the margins of a typewriter are a large enough linear sample if stars and zeros are about equal in number" (p.152).
- **No vacancies; relative positions.** A mover "merely intrudes himself between two others" (p.150), and everyone closes up.
- **Neighborhood.** "everyone defines 'his neighborhood' to include the four nearest neighbors on either side of him" (p.149), so 8 neighbors.
- **Demand.** "everybody wants at least half his neighbors to be like himself ... A star wants at least four of his eight nearest neighbors to be stars ... Including himself, this means that he wants a bare majority, five out of the nine." (p.149)
- **Ends of the line.** "For those near the end of the line the rule is that, of the four neighbors on the side toward the center plus the one, two or three outboard neighbors, half must be like oneself." (p.149) So the neighborhood is truncated to 4–7 neighbors and the rule is still ≥ half. It is a line, not a ring. A ring or an infinite line is mentioned only in a footnote about the 6-neighbor case (p.152).
- **Where a mover goes.** "a dissatisfied member moves to the nearest point that meets his minimum demand—the nearest point at which half his neighbors will be like himself at the time he arrives there. 'Nearest' means the point reached by passing the smallest number of neighbors on the way" (p.150). **Left/right ties are not specified.**
- **Order.** "arbitrarily let the discontented members move in turn, counting from left to right. The star second from the left moves first, the star sixth from the end moves second, and so forth." (p.150)
- **Who moves.** "any originally discontented member who is content when his turn comes will not move after all, and anyone who becomes discontent in the process will have his turn after the 26 original discontents have had their innings." (p.150) This defines rounds: a round is the snapshot of discontents at its start, taken left to right.
- **Definition is dynamic.** "if someone moves in between a man and his next neighbor, the fourth neighbor away ceases to be a neighbor because he is now fifth." (p.150)
- **No anticipation.** "Nobody in this model anticipates the movements of others." (p.150)
- **Stopping.** Stop when nobody is discontent. "There is no guarantee that two rounds will put everybody in equilibrium. One round may do it, more than two may be required." (p.150) **What happens if no satisfactory point exists is not specified** for the line.

The 1969 wording is the same in substance: "moves in either direction to the nearest point (measured in the number he passes on the way) at which half his eight nearest neighbors are the same color"; "move in turn, starting from the left, if they are still discontent when their turns come" (1969 p.490).

### 2.2 Results reported (1-D)

- (b) **Fig.1** (35+/35 0): "Twelve stars and 14 zeros are dissatisfied ... (The expected number is just under 13.)" (p.150). **The 1969 AER version says "11 plusses and 13 zeros"** (p.490) for what looks like the same line. That is an internal discrepancy. My recount of the 1971 line reproduces 12 and 14 exactly (section 5).
- (b) After round 1 (top line of Fig.2), "eight newly discontent individuals". After round 2 everybody is content. "The result is six clusters of like individuals, containing 8, 15, 10, 15, 16 and 6 members respectively, averaging 12 members." (p.150)
- (b) "440 out of 540 neighbors are of the same color, or 81.5%. Counting himself as the ninth member of his neighborhood, everyone lives in a neighborhood in which his own color predominates by an average ratio somewhat greater than five to one. This resulted from individuals' *seeking* a ratio not less than five to four." (p.150)
- (a) "We knew in advance that, if there were an equilibrium, everyone would get to live in a neighborhood at least five-ninths his own color." (p.151)
- (a) "regular alternation of stars and zeros would satisfy everybody ... So would alternating pairs ... Alternating groups of three or four would not meet the condition; but any groups of five or more in alternation meet it. We got groups of about twelve." (p.151)
- (b) **Integration-seekers:** "If people, though not wanting to be in the minority, prefer mixed neighborhoods, only 40 of the 70 managed it at all. Thirty have no neighbors of opposite color." (p.151) (I checked the arithmetic against the Fig.2 clusters and it gives 30.) (c) Those who want some opposite neighbors "can move nearer to the boundary of their own color group, but will never go beyond the boundary ... none of this affects the grouping itself." (p.151)
- (b) **Fig.3** (72 members): "30 discontents; one round of moving" leads to six groups (7, 14, 14, 9, 17, 11) "and the same resulting neighborhood statistics as in the first case." (p.151) My transcription gives 29 under the k=4 rule and exactly 29 under the k=3 rule he also quotes. His 30 may be a miscount, or my transcription may be off by one.
- (c) "Some tabletop experimentation suggests that, with everything else the same, different random sequences yield from about five groupings with an average of 14 members to seven or eight groupings with an average of 9 or 10, six being the modal number of groups and 12 the modal size." (p.151)
- (c) "Similar experimentation suggests that the order of moves makes little difference unless we allow our people to anticipate outcomes and seek either to maximize or to minimize group sizes." (pp.151–152)

### 2.3 Variations of the linear model (pp.152–154)

"Our model has five elements that are readily varied: neighborhood size, demanded percentage of one's own color, ratio of stars to zeros in the total population, rules governing movement, and original configuration." (p.152)

- (b) **Smaller neighborhood (3 on each side, demand half):** "the same general pattern of alternating clusters ... Testing with the two random sequences ... we find 37 initial discontents in the first case, 5 new discontents after the first round of moving, and an end result of ten groups with an average size of 7. The second sequence generates 29 discontents, 3 new ones after the first round, with an end result of seven clusters averaging 10 per cluster." (p.152)
- (c) "Further experiment suggests a mean of 7 or 8 per cluster, or approximately twice the minimum size of cluster that meets the demand (alternating clusters of 4) and with the average person's neighborhood 75% to 80% his own color." (p.152)
- (a) Footnote (p.152): with 6-neighbor neighborhoods, short of clusters of four or more the only satisfying pattern is `...OO+O++O+OO+O++...`, and it "is unstable at the ends: it must run indefinitely in both directions or form a closed curve, else it unravels completely into clusters."
- **Unequal numbers.** He removed 17 of 35 zeros from Fig.1 by die roll ("letting a roll of the die determine the fate of each zero"), leaving 35 stars and 18 zeros (Fig.4). (b) "All the zeros are now discontent, and three of the stars ... all the zeros congregate in the first round, as in Figure 5", which has clusters 15, 18, 20 (p.152). (b) Roughly halving the zeros of the second sequence: "18 of 20 zeros discontent and 2 of the stars. After a round of moving there were still 4 discontented members, and after a second round, 2. After the third round, the top line of Figure 6 was obtained." Two other random deletions give the other lines of Fig.6 (pp.152–153).
- (a) The majority's segregation grows as the ratio becomes extreme. At 4:1 even a regular distribution gives stars ~1/5 opposite neighbors, and the minimum majority clustering satisfying the minority is 4×5=20 (p.153).
- (a)/(c) "What is less immediately apparent ... is that the minority itself tends to become more segregated from the majority, as its relative size diminishes ... the number (frequency) of minority clusters diminishes more than proportionately." The mechanism: fewer "growth nodes" (4+ minority among 8 consecutive). (c) "Even demanding but three neighbors of like color, a 10% minority will form clusters averaging about twice the size of those obtained in Figures 2 and 3. Demanding half, the mean cluster of a 10% minority will contain upwards of 100 if the aggregate population is large enough to sustain any growth nodes at all!" (p.153)
- **Restricted movement** (pp.153–154). This is a minority at 20% or 10% with a travel limit. "Some, probably many, perhaps all, will become unable to move to where their demands are satisfied. We then modify the rule: if a neighborhood half your own color does not occur within the allowed radius of travel, move to the nearest place where three out of eight occur." (c)/(b) "The result is that everybody achieves his desired neighborhood, half or more his own color, without traveling as far as if he and the others had been free to travel! The limitation on travel channels them into the smaller, more frequently occurring, potential clusters ('growth nodes'), which proceed to grow into clusters that more than satisfy them." (a) "Thus travel restrictions imposed on individual movement can be a substitute for *concerted* movement. It can also be a substitute for *anticipatory* movement." **The radius used is not given.**
- Caveat (p.154): "All of this is too abstract and artificial to be a motion picture of whites and blacks ... but it is suggestive."

---------------------------------------------------------------------------------------------------

## 3. The 2-D checkerboard model ("Area Distribution", pp.154–166)

### 3.1 Rules as specified

- **Board.** "divide the area into squares like a checkerboard (but without any alternating colors) and distribute colored chips at random among the squares, leaving some squares blank. One chip occupies one square" (p.154). The worked board is **13 rows × 16 columns = 208 squares** ("for reasons of convenience that I won't go into here", p.156). **The board is bounded, not a torus:** "An actual board, in contrast to a conceptually infinite expanse, has sides and corners; but, then, so probably do most natural areas ... Along the edge of the board a square has only five neighboring squares, and in a corner but three." He suggests "a long and narrow checkerboard six squares wide and twenty long" to study boundaries (p.154).
- **Vacancies.** "In order that people be able to move there must be some vacant spaces; in order that they have significant choice ... quite a few ... It turns out that 25% to 30% vacancies allows fair freedom of movement without making the board too empty." (p.154) Fig.7 actually has **69 stars, 69 zeros, 70 blanks (33.7% vacant)**. I counted this from my transcription; he says "138 randomly distributed stars and zeros" and "equal numbers" (pp.155–156). Later runs use "blank spaces usually equal to about 30% of the total" (p.158).
- **Initial arrangement.** "As in the linear model, I make an initial distribution at random. It might make sense to distribute the blank spaces evenly, but I let them be determined at random, too. (It makes some difference.)" (p.155) How the randomization was physically done on the board is not stated; the 1-D line used a random-digit table. Note that the 1978 book and 1971b reshuffle an integrated board instead (Pancs & Vriend fn.12; Singh et al. p.344).
- **Neighborhood.** "a convenient minimum-sized neighborhood for an individual is his own square plus the eight surrounding; larger 'neighborhoods' can be considered by including the 24 surrounding squares in a 5×5 area" (p.154). "In most of what I'm going to show you, 'neighborhood' has been defined as the eight surrounding squares that, together with one's own square, form a 3×3 square." (p.155) This is the Moore neighborhood, truncated at edges.
- **Demand, absolute or relative.** "Color preferences with respect to one's neighborhood can be defined either in absolute terms—the number of one's own color within the eight surrounding squares—or in relative terms—the ratio of neighbors of one's own color to opposite color among the eight surrounding squares. If all squares were occupied, every absolute number would correspond to a ratio; but because one may have anywhere from zero up to eight neighbors, there are eight denominators and therefore eight numerators to specify" (p.155). **Neighbors means occupied squares; blanks are not neighbors.** This is confirmed by the Fig.7–9 recounts and the "average number of neighbors" statistics.
- **Baseline demand.** "a universal demand that no fewer than half of one's neighbors be of the same color, the discontent moving to the nearest satisfactory vacant square." (p.155) Discontents are those "whose neighbors are less than half of like color" (p.156).
- **Zero neighbors.** A person with no neighbors is not addressed (Pancs & Vriend fn.13 make the same point). Under "less than half ... like" such a person is content (0 < 0 is false).
- **Where a mover goes.** "move to the nearest satisfactory square, with 'nearest' measured by the number of squares one traverses horizontally and vertically." (p.155) That is **Manhattan distance to the nearest vacant square where he would be satisfied**, evaluated at arrival. **Ties are not specified.** "(There is no guarantee that everybody can find a blank space that suits him, but with the numbers we are using now it usually turns out that he can.)" (p.156) **The case where no square suits him is not specified.**
- **Order.** "we need a rule to specify the order in which they move; this part is more complicated than in the linear model. In some cases the order of move was determined merely by position on the board, such as working generally from left to right; it is also interesting to see what happens if all of one color completes its moves before the other color moves. It is possible, of course, to test the sensitivity of the results with respect to the order of moves. **Because what is reported here has all been done by hand and eye, no exact rule for the order of moves has been adhered to strictly.**" (p.155)
- **Process and stop.** "Identify the discontents ... and, in some order, move them to where they are content, continuing to move the newly discontent until the entire board is in equilibrium." (p.156)
- **Invitation.** "The reader can check this for himself in about ten minutes if he has a roll of pennies, a roll of nickels, and a sheet of paper big enough for 16 columns of one-inch squares." (p.156)

### 3.2 Baseline results (equal numbers, demand ≥ 1/2, Moore, 13×16)

- (b) Fig.7 (random start): "zeros on the average have 53% of their neighbors of the same color, stars 46%" (p.156). There are "25 stars and 18 zeros in Figure 7 whose neighbors are less than half of like color" (p.156). There are 13 people with no opposite-color neighbors, "which corresponds exactly to the expected value in an 11×16 matrix with one-third stars, one-third zeros, and one-third blanks" (p.158; "11×16" is as printed).
- (a)/(b) "The *particular* outcome will depend very much on the order in which discontented stars and zeros are moved, the *character* of the outcome not very much." (p.156)
- (b) **Fig.8**: "Working generally from the upper left corner downward and to the right". **Fig.9**: "Working from the center outwards" (p.156). **Fig.10** is Fig.9 with boundaries drawn. He writes "this may be cheating a little".
- (b) Fig.9: "zeros on the average have neighbors who are five-sixths zeros, stars have neighbors who are four-fifths stars. On the average each zero has five neighbors, of whom (not quite) one is a star ... [each star] about four and a half neighbors, one of whom is a zero." With no opposite-color neighbors: 54 people, about 40%, against 10% at random (p.158).
- (b) Fig.8: "the average person lives in a neighborhood that is 90% his own color (89 for zeros, 91 for stars), and two-thirds of both colors have no neighbors of opposite color." (p.158)
- (a) Footnote (p.158): Fig.9's neighbor count equals that of 7×7 same-color blocks (49 each) on an unbounded checkerboard. The 39% with no opposite neighbors corresponds to mono-colored 5×6 groupings (12/30 = 40%). 3×3 blocks give 11%, "almost exactly that expected in a random distribution."
- (a) "Patterning—departure from randomness—will prove to be characteristic of integration, as well as of segregation, if the integration results from choice and not chance." (fn., p.156) "Randomness is not regularity." (p.156)
- **Status of the 2-D generalizations.** "**My samples have been too small, so far, to allow serious generalizations, so I shall formulate hypotheses suggested by what I have done.** Quantitative measures, of course, refer exclusively to an artificial checkerboard and are unlikely to have any quantitative analogue in the living world. *Comparisons* among them, however, such as the effect of reducing or enlarging the size of a minority, may be capable of some extension to that world." (p.158) So everything in 3.3–3.8 is (b) for the specific figure and (c) as a general statement.

### 3.3 Intensity of demand (pp.158–159)

- (c) "If the two colors are equal in number, if neighborhoods are defined as the eight surrounding squares, and if both colors have the same demands for neighbors like themselves, the segregation that results is slight when the demand is for about one-third of one's neighbors like oneself and striking when the demand is as high as one-half." (p.158)
- (b) "the resulting ratios of like to opposite neighbors is upwards of four to one for demands of one-half or more, and less than 1.5 for demands of about one-third." Fig.11 is the one-third case. Its demands are "one like neighbor out of four or fewer, two out of five neighbors or more". The average number of neighbors is 5.5 initially and about 4.5 at equilibrium, so the "average effective preference is therefore in the neighborhood of one-third" (fn., p.159).
- (a)/(c) Three effects of higher demand compound: more initially discontent, more like-color density per move, more induced movement. The resulting segregation is "a rapidly rising function of demands in the range from about 35% to 50%." (p.159)
- (a) With fixed boundaries, demands summing to more than 100% make coexistence impossible. Without fixed boundaries mixtures are possible, "But the degree of flexibility is not great. Therefore we should expect that demands summing to more than one should result in extreme segregation, as apparently they do." (p.159)

### 3.4 Unequal demands (pp.159–160, 163)

- (b) Fig.12 has **77 zeros and 72 stars** (so 59 blanks). "Zeros demand that one out of four or fewer be their own color, two out of five or more; stars demand two of their own color if they have three to five neighbors, three if they have six or seven neighbors, and four out of eight." (pp.159–160) **The stars' demand with 1–2 neighbors is not stated.**
- (b) "Zeros end up with a ratio of 2:1 of neighbors their own color; stars ... show the somewhat higher ratio of 2:5" (p.160). "2:5" is evidently a slip for 5:2: p.163 restates it as "a ratio of 2.5 compared with 2.0".
- (a) "the more demanding end up with a higher proportion of like neighbors, but not much higher" (p.159). Separation is reciprocal, so the two colors' like-neighbor ratios can differ "only by different mean population densities in the neighborhoods of the two colors" (p.160). (b) "the stars are noticeably more compacted than the zeros" (p.160).

### 3.5 Unequal numbers, equal demands (pp.160–161)

- (c) "If we put one of the two colors in minority status, letting it be outnumbered two to one or four to one, greater segregation occurs than with equal numbers, for any given set of demands ... When one of the colors numbers only half the other, demands for about one-third neighbors of like color lead to ratios close to two to one for the minority (and, necessarily, still higher for the majority)." (p.160)
- (b) Fig.13: stars outnumber zeros about 2:1. The demand is "a minimum of two neighbors of like color" (absolute), for an effective demand of about 35%. Zeros go from a random like:unlike ratio of about 1:2 to 2:1, "a fourfold increase". Stars go from 2:1 to 4:1 (pp.160–161).
- (c) With extreme ratios (5:1 or more) "the minority tends to display a phenomenon related to its absolute density rather than its relative density ... nearly everybody in the minority moves ... The result is that the minority forms larger clusters, large enough to cause even a tolerant majority to become locally dissatisfied." (p.161)
- (a) Thought experiment: newcomers of the other color enter one by one into a one-color area. They join the first arrival and cluster, and "the result will be a solid neighborhood of the new color. This result will be independent of the moderateness of the color demands of the newcomers." (p.161)
- (b) **Fig.14** (three panels, one random start): stars outnumber zeros 5:1 and the neighborhood is the 24 surrounding squares (5×5). Zeros demand "an absolute number of but two zeros in the entire 24 surrounding squares". Stars demand "that zeros be no more than one-third of the population in the 24 surrounding squares". Initially only one star is dissatisfied. "The pattern that results from movement is somewhat sensitive to the precise order of movement imputed to the various dissatisfied zeros, of whom there are 11 out of the 15 individuals." The three panels are three moving orders. On 24 neighbors zeros achieve "not quite one-half" like neighbors; counted on 8, "almost three-quarters" (pp.161–163).

### 3.6 Population densities and vacancy (pp.163–164) — "density and vacancy" from the abstract

- (b) Fig.12 (unequal demands): stars average 5.35 neighbors and zeros 4.55. Imputing blanks to territories (14 blanks to stars, 45 to zeros; 10 ambiguous blanks split five and five), "the 72 stars occupy territory comprising 86 spaces altogether, and a population density of .83 ... The 77 zeros occupy territory comprising 122 spaces altogether, with a population density of .63. 'Zero territory' is 37% vacant, 'star territory' only 17%." (p.163)
- (a) The more demanding group "ends up in more homogeneous clusters" and this is "mathematically equivalent to the result that the more demanding ends up with more neighbors, in more densely populated neighborhoods." (p.163)
- (b) Equal demands, unequal numbers (Fig.13): "The minority tends to accumulate in denser neighborhoods than the majority." Neighbors per zero are 6.0 and per star just under 5.0. Zero territory occupancy is about 83% and star territory about 64% (p.164).
- (b) Fig.15: stars:zeros almost 4:1. "Stars demand one star out of four or fewer neighbors, two out of five or more, while zeros demand two out of five or fewer, three out of six or seven, four out of eight." The result is "virtually complete occupancy of 'zero territory' amidst a quite dispersed star population. Stars average 5.1 neighbors apiece; zeros average 6.8, and, given their locations, 7.0 is the maximum." (p.164)
- (a)/(c) He introduces "collective territory" as "a necessary supplement to the 'individual neighborhood' ... even though it enters no one's motivation" (pp.163–164). Caveat: "This density phenomenon is suggestive but not easily related to residential patterns" (p.164). His surfers-and-swimmers aside is that the model "suggests they will become separated into groups but the surfers will enjoy more acreage per capita!"

### 3.7 Size of neighborhood (pp.164–165)

- (c) "Enlarging the area within which a person counts his neighbors attenuates the tendency to segregate, at least for moderate demands and near-equal numbers of the two colors. This observation may not stand up when less tolerant demands, and greater differentials in initial numbers, are put into the larger neighborhoods." That is the whole passage. No figure is given for the equal-numbers larger-neighborhood case. Fig.14 is the only 24-neighbor example and it has unequal numbers.

### 3.8 Congregationist and integrationist preferences (pp.165–166)

- (b) **Congregationist (Fig.16):** "Each individual was assumed to want three neighbors like himself out of eight surrounding spaces (or two out of five, along the edge), and to be indifferent to the presence of the opposite color. That is, opposite neighbors were equivalent to blank spaces." "The degree of 'segregation' compares with that of Figure 10. For the two colors together, like neighbors are just over 75%; and among the two colors, 38% have no neighbors of opposite color." "In achieving an absolute figure of three out of eight like himself ... he separates from the others just as if he had demanded majority status." (p.165) **Numbers of each color in Fig.16 are not stated.**
- **Integrationist (Fig.17).** "In both cases zeros are just over half the number of stars." (p.165)
  - Left panel: upper and lower limits. "among eight neighbors, at least three and at most six like oneself; among seven, at least three and at most five; among six, at least three and at most four; among five, at least two and at most four; among four, either two or three; among three, either one or two; and one out of two." (p.166)
  - Right panel: a ranked preference. "out of eight neighbors, five like oneself is the preferred number but, if the board offers no such choice, then four and six are equally preferred as second choice; failing that, three and seven are equally attractive, with two and eight tied for next place, then one and finally none. Similarly with other numbers of neighbors: half for odd numbers, just over half for even, with second and third choices pairing numbers both above and below the preferred number." (p.166) The phrase "half for odd, just over half for even" is as printed; it is odd given that 5 of 8 is "just over half" of an even count.
  - "Tentative experimentation suggests three phenomena that are not present in the case of purely 'separatist' demands" (all (c), p.166):
    1. "Integration requires more complex patterning than separation; equilibrium is achieved only with a much larger number of moves; and a larger number of individuals move. More individuals may be incapable of being satisfied. And there are problems of the consistency of the integrationist demands, not only of the two colors but of each of them with the overall color ratio in the population."
    2. "If one of the colors is a minority ... the minority is 'rationed.' That is, the patterns have to be 'efficient' in the way many members of the majority can share minority neighbors ... the minority, for example, may be spread out in conspicuous lines rather than clustered in conspicuously convex areas."
    3. "The process of moving produces 'dead spaces.' An area densely settled by either color will be evacuated in its center; neither color will then move into the area, but the boundary is stable because it has contact with the opposite color. The result is that the blank spaces form their own 'clusters' in the final equilibrium."

### 3.9 The overall claim (abstract, p.143)

(a) "The systemic effects are found to be overwhelming: there is no simple correspondence of individual incentive to collective results. Exaggerated separation and patterning result from the dynamics of movement. Inferences about individual motives can usually not be drawn from aggregate patterns. Some unexpected phenomena, like density and vacancy, are generated."

---------------------------------------------------------------------------------------------------

## 4. Bounded-neighborhood model (pp.167–181) and tipping (pp.181–186)

### 4.1 Rules

- "there is a common definition of the neighborhood and its boundaries. A person is either inside it or outside it. Everyone is concerned about the color ratio within the neighborhood but not with any configuration of the colors within the neighborhood." It can also be read as a job, office, university, church, voting bloc, club, restaurant or hospital (p.167).
- "one particular bounded area that everybody, black or white, prefers to its alternatives. He will reside in it unless the percentage of residents of opposite color exceeds some limit. Each person ... has his own limit. ('Tolerance' ...)" Tolerance is comparative, relative to the alternatives (p.167).
- Preferences all go the same direction: an upper limit on the other color and **no lower limit**. "Absolute numbers do not matter, only ratios; there are no economies of scale ... no individual positions within the mix" (p.167).
- Dynamics: "people both leave and return ... People in the area move out if the ratio is not within their color limit; people outside move in if they see that it meets their requirements." "Information is perfect ... People do not, however, know the intentions of others and do not project future turnover." (pp.167–168)
- **Relative speeds are left open:** "we need not stipulate in advance whether whites move in or out more rapidly than blacks do ... in our analysis we can watch and see how they matter." Assumption: the least tolerant leave first and the most tolerant enter first (p.168).
- Tolerance is a cumulative distribution, a "tolerance schedule". **Straight-line example (Fig.18):** 100 whites, with the max acceptable black:white ratio falling linearly from 2.0 (most tolerant) to 0 (least tolerant). The median is 1.0 ("50 among our 100 whites will abide a ratio of black to white of 1.0 or greater"). Blacks have the identical distribution **but number 50** in 1971 (p.168). In **1969 the two groups are equal in number** (1969 p.491).
- Translation: the number of blacks tolerated by the n most tolerant whites is n·R(n). This gives parabolas, "in the same way that a demand curve translates into a total revenue curve" (p.169). Worked numbers: "50 whites can tolerate an equal number of blacks, or 50. Seventy-five can tolerate half their number, 37.5; 25 can tolerate 1.5 times their number, or 37.5. Ninety can tolerate but one-fifth their number, 18; 20 can tolerate 36" (p.169). **Erratum:** a straight line from 2.0 at 0 to 0 at 100 gives R(20)=1.6, so 20 whites tolerate **32**, not 36. The other numbers check.
- A mixture of 25 blacks and 25 whites would be content. There are "10 blacks who could tolerate a ratio of 1.6 to 1, or 16 whites" (p.168).

### 4.2 Results

- (a) Static viability: inside both parabolas means everyone present is content. Other regions mean one or both colors have discontents (p.170).
- (b)/(a) Fig.18 dynamics: "There are only two stable equilibria. One consists of all the blacks and no whites, the other all the whites and no blacks. Which of the two will occur depends on how the process starts and, perhaps, the relative speeds of white and black movement." A mixture "attracts outsiders ... eventually more of just one color ... cumulatively the process causes evacuation of them all." (pp.170–171) 1969: "Up to half of both colors could contentedly coexist at ratios near one to one, but the dynamics of entry prevent any mixture from stabilizing." (1969 p.492)
- (b) **Fig.19:** steeper schedules. The median tolerates 2.5 blacks per white (white minority of 25–30%), the most tolerant tolerate 5:1, and the least tolerant tolerate none; the vertical intercept is 5.0, with equal numbers of 100 each. "in addition to the two stable equilibria ... there is a stable mixture at 80 blacks and 80 whites" (p.171). "As long as half or more of both colors are present—actually, slightly over 40% of both colors—the dynamics of entry and departure will lead to the stable mixture ... Even for very small numbers of both colors present, if the initial ratios are within the slopes of the two curves ... and if neither color tends to enter much more rapidly than the other, the two colors will converge on the 80–80 mixture." Starting from one color, "it would require the concerted entry of more than 25% of the other color." "each of the three equilibria ... is stable against fairly large perturbations." (p.172) I checked the 25% figure and the 80–80 point against the straight-line algebra and both hold.
- (b) **Fig.20:** "The stable equilibrium generated in Figure 19 disappears if the total number of blacks exceeds that of whites or whites exceed blacks by, say, two to one" (p.172).
- (a) **Fig.21:** with the Fig.18 schedules and equal numbers there is no stable mixture. "For straight-line tolerance schedules and equal numbers of the two colors, there is no stable intersection of the two parabolas unless the tolerance schedules have vertical intercepts of 3.0, with median tolerance of 1.5." (p.172) This checks: the symmetric intersection is stable only if the intercept is greater than 3.
- (b) **Limiting numbers, Fig.22:** "If the number of whites in the preferred area is limited to 40 and if the most tolerant 40 are always the first to enter and the last to leave, the curves of Figure 20 are replaced by those of Figure 22, with a stable mixture at 40 whites and a comparable number of blacks." (p.173) With Fig.18 curves "the numbers of both colors would have to be restricted" (Fig.23). Limiting the total but not each color gives "a kind of neutral equilibrium along the overlapped portion of the 45-degree line" (Fig.24), which can drift with turnover (pp.173–174).
- (a) "the limitation on the number of whites that may be present has the same effect in our model as if the whites in excess of that number had no tolerance for any blacks at all." (p.174)
- (a) **Varying tolerance:** "it is not the case that 'greater tolerance' increases the likelihood of a stable mixture—at least, not if 'greater tolerance' means that within a given population some members are statistically replaced by others more tolerant. On the contrary, replacing the two-thirds least tolerant whites in Figure 22 by even less tolerant whites keeps the whites from overwhelming the blacks by their numbers. This would not happen if we made *all* whites less tolerant." (p.174) Fig.25 (broken line) is the schedule that produces a stable mixture when blacks are outnumbered two to one with the Fig.20 curve (p.175).
- **"Varieties of Results"** (pp.175–180). These are summarized "even though the analysis will not be shown":
  1. (a) The only restriction is that the schedule slopes downward and any ray from the origin cuts the curve once. Any number of stable equilibria (0–4+) is possible. "The occurrence of several mixed-color stable equilibria is usually sensitive ... to small changes ... It is the extreme one-color stable equilibria that tend to be least disturbed ... and the occurrence of a single mixed stable equilibrium may be fairly immune to shifts in the curves." Fig.26 has three-tier tolerance with three stable equilibria. Fig.27 has a rectangular hyperbola RN = 0.9 vs. flat R = 1.1 ("precarious" stable mixture).
  2. (a) Limiting one color's numbers is sometimes enough; sometimes both must be limited; if the curves don't overlap no limit helps.
  3. (b) Ratio limits (quotas) may or may not produce stability. In Fig.28 they don't and in Fig.29 they do.
  4. (a) A total-occupancy limit gives a neutral equilibrium (Fig.24).
  5. (a)/(c) "If the two colors have similar tolerance schedules ... the likelihood of a stable mixed equilibrium is greater, the more nearly equal are the numbers of the two colors."
  6. (a) "for a stable mixture, the minority must be the more tolerant of the two groups." For example, if whites outnumber blacks 5:1 in aggregate, a local mix requires "either that blacks be outnumbered by whites or that four-fifths or more of the whites be incapable of abiding equal numbers."
  7. (b)/(a) Two locations, one preferred by both: restricting alternatives can create (Fig.20) or destroy (Fig.19) a mixed equilibrium.
- (a) **Integrationist reinterpretation** (p.180): "the results generated by this analysis do not depend upon each color's having a preference for living separately. They do not even depend on a preference for being in the majority! ... We postulate a preference for mixed living and simply *reinterpret* the same schedules ... The same model fits both interpretations." The one asymmetry is that there is no lower limit on the opposite color. 1969 makes the same point (1969 p.493).
- 1969-only numbers: "Make the least tolerant 60 percent of blacks and whites absolutely intolerant in Figure 1 and a stable equilibrium will occur at forty apiece." With whites limited to half their number and Fig.2 schedules, 100 whites vs 50 blacks, the curves intersect "(at thirty-six blacks)" (1969 p.493). I checked both against the straight-line algebra and both hold.
- **Stated limitations** (p.181): "no allowance for *speculative behavior*, for *time lags* in behavior, for *organized action*, or for *misperception*. It also involves a single area rather than many areas simultaneously affected".

### 4.3 Tipping (pp.181–186)

- Empirical background: Grodzins (1957) says "for the vast majority of white Americans a tipping point exists" and cites 20% Negro as an estimated upper limit. Duncan and Duncan (1957) found no Chicago neighborhood in 1940–50 that was 25–75% white in which succession was arrested or reversed. Mayer (1960), Russell Woods, about 700 homes: "The selling of the third house convinced everyone that the neighborhood was destined to become mixed." A year later 40 houses had been sold, and in another two years the share was over 50%. There is also the Lexington ice cream parlor anecdote.
- Model: a fixed capacity (45-degree line), turnover, potential entrants, alternatives; incumbents are somewhat more tolerant than outsiders. Figs.30–32 show reaction curves and the capacity line.
- (a) "In few if any of these figures is it clear just what we might want to call the 'tipping point.' There are several points at which something discontinuous happens or some cumulative process begins. Furthermore, ... *in none of the cases shown does any important discontinuity necessarily occur at the modal or typical tolerance value*. If 'most Americans can tolerate about 20 percent blacks in their community,' any tipping point or tipping points will tend to occur at quite different percentage figures!" (p.182)
- Distinctions: "in-tipping" (the black curve encloses the all-white point) vs. an all-white stable equilibrium that needs an event to overcome it. There is also the case where enough blacks fill the neighborhood (Fig.30) vs. not enough (Fig.31: all-black and partly vacant, or a mixed equilibrium with excess capacity). In Fig.32 white evacuation stops when the white schedule becomes inelastic (pp.182–184).
- (b)/(c) Subneighborhoods: "If blacks are willing to be 30% of a subneighborhood of 50 houses, they may 'tip in' after the number of black homes reaches 15, even though in a larger neighborhood of 1,000 houses they are only 1½%." (a) "The 'proximity model' of stars and zeros may apply in the small, and the 'bounded area model' in the large." (pp.184–185)
- (c) Speculation and anticipation are aggravating factors. They need a penalty for late departure; the channeling of real-estate activity matters (pp.185–186).
- (c) Open questions: "whether an entire metropolitan area might 'tip' ... whether some major nonresidential unit, say the U.S. Army, could tip" (p.186).

---------------------------------------------------------------------------------------------------

## 5. My checks against his figures (scratchpad `sim/`: `line.py`, `line2.py`, `grid.py`, `grid2.py`, `batch.py`, `minority*.py`)

These are pilot checks, not verified findings. Transcription errors of ±1 cell are possible.

**1-D Fig.1 transcription:** `O+OOO++O+OO++OO+++O++O++OO++OO++OO++O+O+OO+++O++OOOOO+++OOO+OO++O+O++O` (70 people; 35 +, 35 O; `+`=star).
- Initial discontent under "k=4 each side, truncated at ends, like ≥ half": **12 stars, 14 zeros, exactly his 1971 count**. The positions also match his dots. The 1969 count of 11/13 is therefore a slip. With integer counts, "≥ half" and "≥ ceil(half)" are the same, so the end-of-line rule is unambiguous.
- **His first three moves reproduce exactly.** The first discontent "passing six neighbors, inserts himself between the zero who was eighth ... and the star ninth". The next two stars each move "over to the right of" the previous mover.
- **The 4th move diverges.** He says the discontented zero "moves to the left, passing four stars". Under the literal rule a spot 3 stars away already gives him 4 of 8 like. This looks like a hand-simulation slip, or an unstated preference for joining his own cluster.
- The end state under the literal rule is 8 groups (9,14,6,5,6,9,14,7) with right-tie-break, or 6 groups (9,17,12,12,14,6) with left-tie-break. His result is 6 groups (8,15,10,15,16,6). The top line of his Fig.2 (after round 1) matches ours closely in the middle.
- With k=3 his initial discontent counts reproduce exactly: **37** (Fig.1 line) and **29** (Fig.3 line).
- Pilot over 300 random 35/35 lines (k=4, his round and order rules): mean 7.8 groups, **modal 7**, range 4–12, mean size 9.3. Only about 67% of runs fall in his "about five to seven or eight" range. **His modal 6 groups of 12 looks slightly more clustered than the literal rule gives.** With k=3 the mean size is 7.7, matching his "7 or 8 per cluster".
- Pilot minority lines (k=4): a 10% minority demanding 3/8 forms clusters of about 12 (linked if within 4), not the "about twice" of 12 he says. Some agents stay discontent (stuck or cycling). A 10% minority demanding 1/2 in a line of 1000 either forms one cluster of all 100 or finds no growth node. That is consistent with his "upwards of 100 if ... any growth nodes at all!"
- Non-convergence is possible. One left-tie-break run of the Fig.3 line cycles forever with 1 discontent, which is consistent with "no guarantee" of equilibrium.

**2-D Fig.7 transcription** (13×16; `#`=star):
```
O####OOOO__##_OO
O_#OOO_##__#_#O_
#_#OO##_#__O#_##
#__##__O###OO___
O_OO####__#_#_OO
_#O#_OO#__OO__##
#OO#____OOO###__
O_#O_##_#OOO___#
O_#O____##O____#
OO___#__O#OOOO##
_O##OOOO_O##_O##
#_O#O#_OO#O#O_O_
_OO__O#O#OOO__##
```
- Counts: 69 stars, 69 zeros, 70 blanks. Zeros' like-share is 53.5% (his 53%). Stars' like-share is 48% pooled, 51.5% as a mean of individuals (his 46%). No-opposite-neighbor count is **13 (his 13)**.
- Discontent under "like < half of occupied neighbors" is 28 stars and 24 zeros. **He reports 25 and 18.** The "floor(half)" reading gives 17 and 10, so no simple rule reproduces his count. Treat his hand counts as approximate.
- **Fig.8** final (transcribed): 69/69/70, **0 discontent**, like-share 90%, 96/138 (70%) with no opposite neighbors. His figures are 90% and "two-thirds". **Fig.9** (my transcription has 71 stars, so it's off by 2 somewhere): 0 discontent, like-share about 80%, 56 with no opposite neighbors (his 54). Both final boards are equilibria under the Moore + blanks-excluded + ≥½ reading.

---------------------------------------------------------------------------------------------------

## 6. Specified vs. unspecified (summary for implementation)

| Element | 1-D line | 2-D checkerboard | Bounded neighborhood |
|---|---|---|---|
| Space | finite line, no gaps, relative insertion | bounded 13×16 board (no torus); edges and corners have 5 and 3 neighbors | one area plus an unnamed outside |
| Population | 35/35 from a random-digit table; 72 in 2nd example; minorities by die-roll deletion | 69/69 + 70 blanks in Fig.7; others equal or 2:1+, about 30% blank | 100 W / 50 B (1971), 100/100 (1969), variants |
| Initial | random | random, blanks random ("It makes some difference") | stated starting points |
| Neighborhood | 4 each side (3 in variant); truncated at ends | Moore 8 (24 in Fig.14); only occupied squares count | whole area |
| Demand | ≥ half like | ≥ half like (baseline); absolute or relative tables per figure; integrationist bands and rankings; congregationist absolute 3 | tolerance schedule (max opposite:own ratio), heterogeneous |
| Destination | nearest point by count of people passed, at arrival | nearest vacant square by Manhattan distance where satisfied at arrival | in or out |
| Ties | **unspecified** | **unspecified** | least tolerant leave first, most tolerant enter first |
| No acceptable spot | **unspecified** | "usually turns out that he can"; **unspecified** | n/a |
| Order | left to right; the round is a snapshot of discontents; newly discontent wait for the next round; skip if content at turn | "no exact rule ... adhered to strictly"; examples: left-to-right/top-down, center outwards, one color first | relative speeds are free parameters |
| Stop | nobody discontent | "entire board is in equilibrium" | stable equilibrium |
| Zero neighbors | n/a | **unspecified** (content under "fewer than half" reading) | n/a |

**Epstein & Axtell variant differences** (our current engine): von Neumann 4 vs Moore 8; random acceptable site vs nearest (Manhattan) acceptable; torus vs bounded board; any board size vs 13×16; random or asynchronous activation vs position-ordered sweeps. A faithful "Schelling 1971" default would be: 13×16 bounded board, 69+69+70 blanks (or 1/3 each), Moore neighborhood with blanks excluded from the denominator, demand ≥1/2, discontents move to the Manhattan-nearest satisfying vacancy in a snapshot sweep (top-left→bottom-right as in Fig.8), with named switches for tie-breaking, sweep order (center-out, one-color-first, random), and the no-spot rule (stay).

---------------------------------------------------------------------------------------------------

## 7. Follow-up papers

### 7.1 Pancs & Vriend (2007), J. Public Econ. 91:1–24

- **Variant.** 2-D: bounded board (they argue tori are unrealistic, "No results depend crucially on this choice"), Moore neighborhood, 40% of each type and 20% empty. They mostly use a **5×5 board with 10 of each and 5 empty**, and also 100×100 with 4000 each. 1-D: a **ring** with k=4 each side (10+10, 100+100). Dynamics: at each period **one agent is chosen uniformly at random** and plays a **global best response**. Any empty cell is eligible (2-D) or any insertion point (1-D). There is **no inertia**: indifferences, including staying put, are broken uniformly at random, so content agents also move. There is **no preference for nearby positions**. Utilities, over x = % unlike: *flat* (Schelling: indifferent up to 50% unlike, then drop); *p50* (rises to a peak at 50/50, cutoff at 50%); *p100* (symmetric single peak at 50%, no cutoff); *spiked* (only 50/50 is good). Empty neighborhoods are least preferred. Measures: clusters (lateral links through "uncontended" blank zones), switch rate, distance, mix deviation, share, ghetto rate.
- **Claims with numbers.**
  - 2-D 5×5: random allocations average **7.82 clusters**. After 100,000 periods (1000 runs): flat **2.10** (91% complete segregation), p50 **2.04** (98%), p100 **4.99**. Exhaustive MNE counts: 430,110 flat, 2,880 p50 (64% of them completely segregated), 387,954 p100 (36,482 strict). Full table in their Table 2.
  - 100×100: starting from just over 2000 clusters, p100 goes to about 1700 while flat and p50 segregate strongly and quickly.
  - 1-D ring 10+10: **every utility function ends in complete segregation (2 clusters)** in all 1000 runs, although peaked and spiked MNE are all perfectly integrated. Prop. A1 proves complete segregation is the unique recurrent class for any ring size (m>k). 100+100 ring: flat reaches complete segregation within 25k periods; peaked and spiked reach 6 clusters after 50M periods.
  - Key interpretive claim: in 2-D the driver is the asymmetry at the cutoff ("an agent favors his own ghetto over an unlike ghetto"). In 1-D even strictly integrationist, unbiased preferences segregate. "the one-dimensional and two-dimensional versions ... are in fact two qualitatively very different models."
  - Pushback on Schelling p.151: "We could have surmised that our rules of movement would lead to equilibria" holds for flat utility but not for peaked or spiked.
- **Relation to Schelling.** They recast his rules as myopic best response and drop his nearest-move rule, inertia and order. They report (in their 2003 working paper) that adding inertia, simultaneity or a torus changes nothing qualitative. Their "flat" dynamics are **not Schelling's dynamics**: content agents keep moving among equally good sites, which drifts toward segregation.
- **Reproduction feasibility.** Easy on a grid engine: global random best response, a "no inertia" switch, 4 utility shapes, 5×5 and larger boards. The 1-D ring needs a separate line or ring topology with insertion moves. The exhaustive MNE count on 5×5 is optional but feasible (4.9e9 allocations/2 is heavy; skip).

### 7.2 Gauvin, Vannimenus & Nadal (2009), Eur. Phys. J. B (arXiv 0903.4694)

- **Variant.** L×L lattice (L=50; figures at L=100) with **free (bounded) boundaries** and Moore neighborhood. An agent is satisfied if unlike neighbors ≤ T × occupied neighbors, so T is the max unlike fraction (Schelling's "≥ half like" is T=1/2). The two groups are equal. Dynamics: a **randomly chosen agent (satisfied or not)** moves to a **random vacancy where it would be satisfied** (long-range); if none exists it stays. Time is Monte Carlo steps. Measures: segregation coefficient s = 2Σn_c²/(L²(1−ρ))², with clusters by 4-neighbor adjacency and averaged over 30,000 steps after equilibrium. They also use the density of vacancies unacceptable to a group, energy, specific-heat and susceptibility analogues, and renormalization at high ρ.
- **Claims with numbers.** They give a phase diagram in (vacancy ρ, tolerance T):
  - **frozen** state for T < T_f(ρ) (nobody can move). T_f falls with ρ, e.g. 3/8–1/2 at ρ=2–6% and 1/5–1/4 around ρ=24–40%. The line vanishes above ρ≈46%, and near it the outcome depends on move order (their Table 1).
  - **segregated** (s≈1, two clusters) for T_f < T < T_c. T_c is nearly constant, and "the segregated phase occupies a large domain (up to a tolerance T as high as 3/4)".
  - **mixed** above T_c. The transition is abrupt (first-order-like) for ρ<26%, fluctuating for 26–48%, and continuous near ρ≈50–56%, with "diluted segregation" at high ρ (beyond the 0.5927 percolation threshold).
  - At ρ=5%, T=1/2, L=100 the system is completely segregated within ~10 steps and then coarsens.
  - Without satisfied-agent moves (Schelling's rule) there is a Lyapunov function (Blume–Emery–Griffiths energy with K=2T−1).
  - Interpretive claim: the abrupt mixed→segregated transition "could be interpreted as the tipping point".
- **Relation.** A statistical-physics map of Schelling's two parameters he flagged: demand and vacancy. It uses random rather than nearest destinations and lets satisfied agents move (noise). Note that it says Schelling used "less than 1/3" (the 1978 framing); the 1971 baseline is 1/2.
- **Feasibility.** Very feasible: bounded board, Moore, random-acceptable-vacancy movement (like our E&A rule but with Moore and any-agent activation), and a sweep over ρ and T. The s coefficient is simple. Discrete T values (k/n) matter, so reproduce with fractions.

### 7.3 Singh, Vainchtein & Weiss (2009), Demographic Research 21(12):341–366

- **Variant.** N×N **torus**, Moore neighborhood. The threshold is **absolute**: happy if ≥ T of the 8 surrounding cells are like, so vacancies count against. T ∈ {3,4,5}; T=3 is their reading of Schelling's (1978/2006) setup. Movement **alternates** an unhappy B and an unhappy R, each moved to a **random vacant site with ≥T like neighbors**, until no allowable switch. For T=5 they also allow direct R–B swaps. Initial state: a checkerboard (maximally integrated) with v/2 of each type removed at random and agents permuted in two 3×3 blocks. They say random starts give similar finals except at small v. N = 8, 50, 100, 200; v = 2–33%; 100 runs per cell.
- **Claims with numbers.**
  - "for the values of disparate neighbor comfort threshold used by Schelling, the striking global aggregation Schelling observed is strictly a small city phenomenon." At T=3, N=8 most runs end in two clusters. At N=100 the cluster count rises from **22 (v=24%) to 55 (v=33%)**, and N_C ~ v³ (fitted slope 2.86).
  - T=3 finals are sparse because of the checkerboard's "super-stability" (every agent has a spare like neighbor), so Schelling "required a large density of vacant spaces v (33%)".
  - T=4 gives compact clusters with N_C linear in v (slope 0.89) and scale L growing as v falls, reaching city-wide (L=N) at v≈2%. About 40%+ of agents have only like neighbors, against about 10% at T=3.
  - T=5 gets stuck without swaps. With swaps there is one big cluster per type plus unhappy stragglers: about 10% unhappy at v=10% and about 33% at v=30%.
  - A perimeter (adjusted R–B and agent–vacancy contacts) is a Lyapunov function, so runs always converge. Results for N=50, 100 and 200 are qualitatively the same.
  - Mean switches: 3596 (T=3, v=33%), 5192 (T=4, v=33%), 5573 (T=4, v=2%), 4422 (T=5, v=2%).
- **Relation and fairness.** Their "Schelling" is the 1978 book's 8×8 deleted-checkerboard with absolute 3-of-8 (vacancies count against), which is roughly a one-third demand. Schelling 1971 already said one-third demand gives "slight" segregation (p.158). His striking 1971 cases use ≥1/2 of occupied neighbors on a random 13×16 start. So Singh et al. challenge the 1978 version most directly. At T=4, their own large-city results still show strong (mesoscale to macroscale) aggregation.
- **Feasibility.** Feasible: torus, Moore, absolute threshold counting blanks as non-like (a named switch: "blanks in denominator"), random acceptable destination, deleted-checkerboard init, board-size sweep 8→200. Their Lyapunov perimeter is a cheap measure.

### 7.4 Bruch & Mare (2008 CCPR working paper; Oxford Handbook of Analytical Sociology chapter "Segregation Processes", rev. Oct 2007)

- **What this file is.** It's a review chapter, not the 2006 AJS article ("Neighborhood Choice and Neighborhood Change"). It presents a conditional-logit residential choice model, p_ij ∝ exp(β·Z_j + ...), with the choice set restricted by radius, price or discrimination. It contrasts two decision rules with "the same average level of tolerance" (their Fig.1): **threshold** p ∝ exp(1{Z ≥ 0.5}) and **continuous** p ∝ exp(Z), where Z is the own-group proportion.
- **Claims (summarizing their 2006 AJS).** "Higher levels of segregation occur when agents' preferences follow a threshold function. When agents make finer-grained distinctions ... preferences alone may not lead to segregation because the segregating effects of changes in the size of the population at risk of entering a neighborhood are offset by a corresponding change in that neighborhood's desirability." They also claim that with neighborhood size fixed, "the same preference function can produce a low level of segregation in a large population, but a high level of segregation in a small population". The chapter gives no numbers or grids, so this document contains nothing numeric to reproduce.
- **The debate (not in this file).** From memory, to verify before citing: van de Rijt, Siegel & Macy (2009, AJS 114(4):1166–1180, "Neighborhood Chance and Neighborhood Change: A Comment on Bruch and Mare") reanalyzed B&M's model. They reported that the lower segregation under continuous preferences was an artifact, attributed to an implementation error and/or the level of randomness in the logit choice, rather than to continuity itself. They found continuous and threshold preferences both segregate once choice is less noisy, and that even integrationist (e.g. multi-peaked) preferences can yield segregation. Bruch & Mare replied in the same issue and acknowledged an error while defending parts of their conclusion. **I have not read either piece; get both before scripting an episode on this.**
- **Relation and feasibility.** B&M replace Schelling's deterministic threshold with probabilistic (logit) choice. The crux of the debate is the noise scale β. With β=1 as written, even the threshold function gives only an e≈2.7 odds ratio across the threshold, which is very noisy compared with Schelling's hard rule. **A fair reproduction needs a β sweep** for both functional forms. It is feasible on a grid as a "utility-based probabilistic move" switch: each mover samples a vacancy with probability ∝ exp(β·u(Z)), using local-window Z. It is fair only if the neighborhood definition (B&M used larger areas/grids) and β are named switches.

### 7.5 Zhang (2004), J. Math. Sociol. 28:147–170, "A Dynamic Model of Residential Segregation"

- **Variant.** A torus lattice with H neighbors (Moore, H=8, in simulations) on **30×30 with 400 black, 400 white, 100 vacant**. **Endogenous price:** P_i = B_i + W_i (occupied neighbors of location i; derived from a linear "natural vacancy rate" pricing rule and normalized). Everyone has income Y. **Whites:** U = Y + y·W − B, with y = p − 1 > −1 and y=1 in simulations. **Blacks are color-neutral:** U = Y − W − B, so they want cheap, emptier locations. **Moves:** each period a random pair of locations is drawn. vacant+occupied means the occupant may move. Two occupied means the two may **swap**, decided on the *sum* of their utilities (so implicitly with side payments). All five move types are allowed, including same-color swaps. **Choice rule:** log-linear (logit) with noise β, so P(move) = e^{βu(move)}/(e^{βu(move)}+e^{βu(stay)}). Because ΔSW = 2·Δu(mover), this is a **potential game**.
- **Claims.**
  - Prop.1: the potential-maximizing states are stochastically stable (Young 1998 Thm 6.1).
  - Claim 2 and Prop.2: with few vacancies, "if blacks are color-neutral and whites have a slight preference for like-color neighbors, then, in the long run: (i) residential segregation is observed most of the time; (ii) the rate of vacancy is higher in black neighborhoods than in white neighborhoods; and (iii) whites pay more than blacks do for equivalent housing."
  - Simulation 1 (y=1, β=2, Y=10, random start): complete segregation; black areas hold the vacancies; a price gap (APB < APW) emerges. Expected waiting time to potential > 7800 falls sharply in β and is flat beyond β≈1. Vacancy share has little effect on speed *because swaps are allowed*; without swaps, fewer vacancies slow segregation markedly.
  - Simulation 2: start from square ghettos with high black prices and low black vacancies (pre-1968). Free mobility gives white retreat, ghetto expansion, persistent segregation, and **reversal** of price and vacancy differentials.
  - Simulation 3 ("integrationist" whites only): U_w = Y − P + p1·min{W,x} − p2·max{0, W−x} with x=5 (≈60% white ideal), p1=2, p2=0.5, blacks neutral. The result is severe but incomplete segregation. "only 12.5 percent of whites have 5 white neighbors; 77.3 percent of them have more than 5".
  - Conclusion: "Without the housing market, segregation will not emerge if individuals are initially assigned to locations randomly" (under his asymmetric preferences).
- **Half-and-half ideal.** **Not in this paper.** The both-groups-prefer-50/50 case is Zhang (2004b), "Residential segregation in an all-integrationist world", J. Econ. Behav. Organ. 54:533–550. This paper only cites it ("Zhang (2004) further shows that segregation could emerge even if both blacks and whites prefer mixed neighborhoods"). Pancs & Vriend cite the JEBO paper, not this one.
- **Relation.** It builds on Young's (1998) 1-D stochastic-stability model in 2-D. The mechanism is asymmetry (one group's own-type bias priced into housing), which parallels P&V's finding that in 2-D the key driver is the asymmetric cutoff. Unlike Schelling, it has no thresholds, uses swaps with transfers, and is noisy and global (random pair, not nearest).
- **Feasibility.** Moderately feasible. It needs a price field (occupied-neighbor count), swap moves between occupied cells with summed-utility acceptance, and a logit β. Grid, torus and Moore are all standard. Prop.2's three observables (segregation, vacancy gap, price gap) are easy to measure. The JEBO half-and-half paper would be needed for the "everyone prefers integration" 2-D episode.

---------------------------------------------------------------------------------------------------

## 8. Candidate claims to test over 20 seeds (Schelling 1971 as default)

1. 1-D, 70 random, k=4, ≥½, his order: clusters ~6 (mode), size ~12; 5–8 groups. Pilot: mode 7, size ~9.3.
2. 1-D k=3: mean cluster 7–8, neighborhoods 75–80% own. Pilot: 7.7.
3. 1-D minority 10%: 3/8 demand gives clusters "about twice" those of Figs 2–3; ½ demand gives ≥100 or none. Pilot: ~12 (fails?) and all-or-none (holds).
4. 1-D restricted travel with fallback to 3-of-8: everyone ends ≥½ own (radius unspecified; sweep it).
5. 2-D 13×16, 1/3 each, Moore, ≥½, nearest-Manhattan: like-share ~80–90%, 40–67% with no opposite neighbors; the character of the outcome is order-insensitive (compare sweep orders).
6. Demand 1/3 vs 1/2: like:unlike < 1.5 vs > 4; segregation rises steeply between 35% and 50%.
7. Unequal demands: the more demanding group ends denser (Fig.12: 17% vs 37% vacant territory).
8. 2:1 numbers, equal absolute-2 demand: the minority's like:unlike goes from ~1:2 to 2:1; the minority is denser (Fig.13: 83% vs 64% occupancy).
9. 24-neighborhood attenuates segregation for moderate demands and equal numbers (never shown by him; a genuine test).
10. Congregationist 3-of-8, indifferent to opposite: like share >75%, 38% no-opposite.
11. Integrationist bands (3–6 of 8, etc.): more moves and movers, unsatisfiable agents, minority in lines, "dead spaces" (blank clusters).
12. Bounded neighborhood (ODE or agent dynamics): Fig.18 has only the all-one-color equilibria; Fig.19 has a stable 80–80 basin (≥~40% of both present; >25% concerted entry needed); intercept-3.0 threshold; Fig.22 has a 40-white cap giving a stable mix; "less tolerant 2/3" beats "all less tolerant"; the minority must be more tolerant (result 6).
