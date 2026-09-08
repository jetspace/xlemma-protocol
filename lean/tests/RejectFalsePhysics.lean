import XLemma.Physics.Oscillator

-- Forward Euler does not conserve oscillator energy at this concrete step.
example : (1 : Rat) * 1 + (-1) * (-1) = 1 := by
  grind
