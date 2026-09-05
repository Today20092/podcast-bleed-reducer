# Use one-pole gain transitions with separate open and close controls

CleanSlate uses the prior Python automixer's exponential one-pole target-chasing behavior instead of finite linear ramps. Separate session-wide Open time and Close time constants preserve smooth asymmetric recovery and attenuation, including retargeting from the current gain; the same resulting envelope drives the GUI preview, Processed and Removed audition, and final render.
