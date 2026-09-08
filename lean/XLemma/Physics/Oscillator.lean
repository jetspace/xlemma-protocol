import Std

/- Exact rational, nondimensional, unit-mass implicit-midpoint model.
   These are discrete mathematical identities, not experimental evidence. -/
namespace XLemma.Physics

theorem midpoint_work_energy (x y v w h k d : Rat)
    (hx : 2 * (y - x) = h * (v + w))
    (hv : 2 * (w - v) = -h * (k * (x + y) + d * (v + w))) :
    2 * ((k * y * y + w * w) - (k * x * x + v * v)) =
      -h * d * (v + w) * (v + w) := by
  grind

theorem forward_euler_energy_growth (x v h : Rat) :
    (x + h * v) * (x + h * v) + (v - h * x) * (v - h * x) =
      (1 + h * h) * (x * x + v * v) := by
  grind

end XLemma.Physics
