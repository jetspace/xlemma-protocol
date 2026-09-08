import XLemma.Physics.Oscillator

namespace XLemma.Physics

/-- Author-operated downstream reuse of the general balance at zero damping. -/
theorem midpoint_conserves_energy (x y v w h k : Rat)
    (hx : 2 * (y - x) = h * (v + w))
    (hv : 2 * (w - v) = -h * k * (x + y)) :
    k * y * y + w * w = k * x * x + v * v := by
  have balance := midpoint_work_energy x y v w h k 0 hx (by grind)
  grind

end XLemma.Physics
