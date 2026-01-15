# Interval Arithmetic Paper Review

*Author: Brendon Bown*

*Date written: November 13, 2025*

As a part of developing the evaluator for Oneil, I read the paper
[*Interval Arithmetic: from Principles to Implementation*](https://fab.cba.mit.edu/classes/S62.12/docs/Hickey_interval.pdf)
by Q. Ju T. Hickey and M.H. van Emden. Here are my takeaways from
it.

## Interval Definition

Section 2 defines an interval as **closed** (as in, end-point inclusive)
and **connected** (as in, there are no holes). It also states that there
are only four "forms" for sets that form intervals:

  - $\{ x \in \mathcal{R} | a \leq x \leq b \}$
  - $\{ x \in \mathcal{R} | x \leq b \}$
  - $\{ x \in \mathcal{R} | a \leq x \}$
  - $\mathcal{R}$

Later, Section 4.1 defines a syntax for these:

  - $\langle a , b \rangle = \{ x \in \mathcal{R} | a \leq x \leq b \}$
  - $\langle -\infty , b \rangle = \{ x \in \mathcal{R} | x \leq b \}$
  - $\langle a , \infty \rangle = \{ x \in \mathcal{R} | a \leq x \}$
  - $\langle -\infty , \infty \rangle = \mathcal{R}$

These sections helped solidify exactly *what* an interval was in my
head. Specifically, I hadn't previously considered the fact that an
interval should be *closed*.


## Inclusion Property

Section 3.1 defines an **inclusion property**. In essence, this property
says the following. Let $E$ be an expression with variables $v_1,...,v_n$.
Let $p$ be the result of evaluating $E$ with values $a_1,...,a_n$
replacing $v_1,...,v_n$. Let $I$ be the result of evaluating $E$ with
the values $I_1,...,I_n$ replacing $v_1,...,v_n$, where
$a_1 \in I_1,...,a_n \in I_n$. Then the property $p \in I$ should hold.

In other words, if every interval in an interval expression is replaced
with a value within the interval, the result of evaluating that expression
should be within the result of evaluating the interval expression.

This is the approach that I took with the "correctness tests" in my
[interval arithmetic experiment](/docs/research/2025-11-12-interval-arithmetic-analysis/) previously, although my approach
was less formalized than here.

My main takeaway from this section is that this property would be a great
property to fuzz test.


## Classification

Section 4.3, and the corresponding Figure 1, define how intervals can be
classified. Specifically, they define the following:

Class of $\langle a , b \rangle$ | at least one negative | at least one positive | signs of endpoints
---------------------------------|-----------------------|-----------------------|------------------------
Mixed ($M$)                      | yes                   | yes                   | $a < 0 \wedge b > 0$
Zero ($Z$)                       | no                    | no                    | $a = 0 \wedge b = 0$
Positive ($P$)                   | no                    | yes                   | $a \geq 0 \wedge b > 0$
Positive0 ($P_0$)                | no                    | yes                   | $a = 0 \wedge b > 0$
Positive1 ($P_1$)                | no                    | yes                   | $a > 0 \wedge b > 0$
Negative ($N$)                   | yes                   | no                    | $a < 0 \wedge b \leq 0$
Negative0 ($N_0$)                | yes                   | no                    | $a < 0 \wedge b = 0$
Negative1 ($N_1$)                | yes                   | no                    | $a < 0 \wedge b < 0$

I noticed that this is the same classification that was used in the
[`inira` crate](https://docs.rs/inari/2.0.0/src/inari/classify.rs.html#34-43)
(with the addition of an "empty" class), and I'm curious if they got the
classification from this paper or if this is a widely known
classification.

Regardless, this classification is essential for multiplication and
division.


## Operations

Section 4.5-4.7 define operations for intervals. These definitions will be
helpful for implementing the arithmetic.

### Interval Addition and Subtraction

Theorem 4 in Section 4.5 defines addition as

$$
\langle a, b \rangle + \langle c, d \rangle = \langle a + c, b + d \rangle
$$

and subtraction as

$$
\langle a, b \rangle - \langle c, d \rangle = \langle a - d, b - c \rangle
$$

Simple enough.


### Interval Multiplication

Interval multiplication gets a little more complicated. Theorem 6 and
Figure 3 in Section 4.6 define the operation of multiplication as the
following table:

Class of $\langle a,b \rangle$ | Class of $\langle c,d \rangle$ | Left endpoint of $\langle a,b \rangle * \langle c,d \rangle$ | Left endpoint of $\langle a,b \rangle * \langle c,d \rangle$
-------------------------------|--------------------------------|--------------------------------------------------------------|-------------------------------------------------------------
Positive ($P$)                 | Positive ($P$)                 | $a * c$                                                      | $b * d$
Positive ($P$)                 | Mixed ($M$)                    | $b * c$                                                      | $b * d$
Positive ($P$)                 | Negative ($N$)                 | $b * c$                                                      | $a * d$
Mixed ($M$)                    | Positive ($P$)                 | $a * d$                                                      | $b * d$
Mixed ($M$)                    | Mixed ($M$)                    | $min(a*d, b*c)$                                              | $max(a*c, b*d)$
Mixed ($M$)                    | Negative ($N$)                 | $b * c$                                                      | $a * c$
Negative ($N$)                 | Positive ($P$)                 | $a * d$                                                      | $b * c$
Negative ($N$)                 | Mixed ($M$)                    | $a * d$                                                      | $a * c$
Negative ($N$)                 | Negative ($N$)                 | $b * d$                                                      | $a * c$
Zero ($Z$)                     | Any ($P$,$M$,$N$,$Z$)          | $0$                                                          | $0$
Any ($P$,$M$,$N$,$Z$)          | Zero ($Z$)                     | $0$                                                          | $0$


### Interval Division

Interval division is the most complicated of the four operations. For
example, $\langle 1,1 \rangle / \langle -\infty,1 \rangle$ produces

$$
\{x / y | x \in \langle 1,1 \rangle , y \in \langle -\infty , 1 \rangle , y \neq 0 \}
$$

$$
= \{x / y | x = 1, y \leq 1, y \neq 0 \}
$$

$$
= \{x | x < 0 \} \cup \{x | 1 \leq x \}
$$

which is not a valid interval, since it is neither closed nor connected.

Section 4.7 suggests a couple solutions to this. The solution that I
think is most appropriate for Oneil is to use the least interval
containing the result. So, in the case of above, that would be
$\langle -\infty, \infty \rangle$.

Theorem 8 and Figure 4 provide the results of the operation of division.
The following is an adaptation of the table that handles all of the
cases as described in the paper.

Class of $\langle a,b \rangle$ | Class of $\langle c,d \rangle$ | $\langle a,b \rangle / \langle c,d \rangle general formula$
-------------------------------|--------------------------------|------------------------------------------------------------------------------------------
Zero ($Z$)                     | Any Non-Zero ($P$,$M$,$N$)     | $\langle 0,0 \rangle$
Any ($P$,$M$,$N$,$Z$)          | Zero ($Z$)                     | $\emptyset$
Positive1 ($P_1$)              | Positive1 ($P_1$)              | $\langle a / d , b / c \rangle \backslash \{0\}$
Positive1 ($P_1$)              | Positive0 ($P_0$)              | $\langle a / d , \infty \rangle \backslash \{0\}$
Positive0 ($P_0$)              | Positive1 ($P_1$)              | $\langle 0 , b / c \rangle$
Positive0 ($P_0$)              | Positive0 ($P_0$)              | $\langle 0 , \infty \rangle$
Mixed ($M$)                    | Positive1 ($P_1$)              | $\langle a / c , b / c \rangle$
Mixed ($M$)                    | Positive0 ($P_0$)              | $\langle -\infty , \infty \rangle$
Negative0 ($N_0$)              | Positive1 ($P_1$)              | $\langle a/c , 0 \rangle$
Negative0 ($N_0$)              | Positive0 ($P_0$)              | $\langle -\infty , 0 \rangle$
Negative1 ($N_1$)              | Positive1 ($P_1$)              | $\langle a / c , b / d \rangle \backslash \{0\}$
Negative1 ($N_1$)              | Positive0 ($P_0$)              | $\langle -\infty , b / d \rangle \backslash \{0\}$
Positive1 ($P_1$)              | Mixed ($M$)                    | $(\langle -\infty , a / c \rangle \cup \langle a / d , \infty \rangle) \backslash \{0\}$
Positive0 ($P_0$)              | Mixed ($M$)                    | $\langle -\infty , \infty \rangle$
Mixed ($M$)                    | Mixed ($M$)                    | $\langle -\infty , \infty \rangle$
Negative0 ($N_0$)              | Mixed ($M$)                    | $\langle -\infty , \infty \rangle$
Negative1 ($N_1$)              | Mixed ($M$)                    | $(\langle -\infty , b / c \rangle \cup \langle b / d , \infty \rangle) \backslash \{0\}$
Positive1 ($P_1$)              | Negative1 ($N_1$)              | $\langle b / d , a / c \rangle$
Positive1 ($P_1$)              | Negative0 ($N_0$)              | $\langle -\infty , a / c \rangle \backslash \{0\}$
Positive0 ($P_0$)              | Negative1 ($N_1$)              | $\langle b / d , 0 \rangle$
Positive0 ($P_0$)              | Negative0 ($N_0$)              | $\langle -\infty , 0 \rangle$
Mixed ($M$)                    | Negative1 ($N_1$)              | $\langle b / d , a / d \rangle$
Mixed ($M$)                    | Negative0 ($N_0$)              | $\langle -\infty , \infty \rangle$
Negative0 ($N_0$)              | Negative1 ($N_1$)              | $\langle 0 , a / d \rangle$
Negative0 ($N_0$)              | Negative0 ($N_0$)              | $\langle 0 , \infty \rangle$
Negative1 ($N_1$)              | Negative1 ($N_1$)              | $\langle b / c , a / d \rangle$
Negative1 ($N_1$)              | Negative0 ($N_0$)              | $\langle b / c , \infty \rangle \backslash \{0\}$


## Rounding

Section 5.3 makes an interesting point about rounding. Specifically, it
points out that the best strategy for maintaining correctness with
intervals is to round *outward*. In other words, when rounding is needed,
round the minimum down and the maximum up.

Rounding is helpful for representing an infinite set of values (real numbers)
in a finite system (floating-point numbers). Theorem 13, Figure 6 and Figure 7
provide a guide on how rounding should be applied to arithmetic operations.

## Remaining Questions

While this provided a solid foundation on which to start, there are
many other operations that I will need to delve into further to
determine how they should operate.

1. **Modulo** - in the operation `x % y`, should `y` be required to
   be a scalar value, rather than an interval? If `y` is an interval,
   how would I calculate the resulting interval?

2. **Exponential** - should the exponent be allowed to be an interval?
   Is it possible to give a useful answer if it is? Would the
   implementation complexity be worth the cost? Are there any situations
   where an interval exponent would be useful? What if I allow the
   exponent to be an interval if the base is a scalar?

Another consideration is whether I should represent all values as intervals,
or whether I should start out in "scalar mode", then switch to "interval mode"
when it's needed. The second option would likely be beneficial for those
who never use intervals in their models, as they wouldn't have to pay the
overhead associated with interval arithmetic.

## Conclusion

This paper was very helpful in laying out the basics of interval
arithmetic and its implementation. It defined the properties of
an interval, including the inclusion property that will be
useful for fuzz testing. In addition, it contained useful definitions
for four basic operators (addition, subtraction, multiplication,
and division) and addressed how rounding can be used effectively to represent
possibly infinite values in a finite system. All together, this information
will improve the quality of interval arithmetic in Oneil.
