import ch.ethz.icr.growlab.collector.PowerLawSeriesCollector;
import ch.ethz.icr.growlab.model.geosim2.Geosim2Lab;
import ch.ethz.icr.growlab.model.geosim2.Geosim2Model;
import ch.ethz.icr.growlab.model.geosim2.War;
import ch.ethz.icr.growlab.simulator.Simulator;
import ch.ethz.icr.growlab.variable.AbstractVariable;
import java.lang.reflect.Field;
import java.lang.reflect.Proxy;

/** Isolated execution bridge for unchanged archived binaries; no model rules. */
public final class GeoSimReferenceProbe {
    private static Object field(Object owner, String name) throws Exception {
        Field f = owner.getClass().getDeclaredField(name);
        f.setAccessible(true);
        return f.get(owner);
    }

    private static void snapshot(Geosim2Lab lab, Geosim2Model model) throws Exception {
        System.out.println("REFERENCE {\"type\":\"snapshot\",\"time\":"
            + lab.getCurrentTime().getValue(model).intValue() + ",\"sovereigns\":"
            + field(model, "numSovActors") + ",\"active_wars\":" + War.getNum()
            + ",\"queued\":" + War.getNumWars() + "}");
    }

    public static void main(String[] args) throws Exception {
        int periods = Integer.parseInt(args[0]);
        if (periods < 1 || periods > 10500) throw new IllegalArgumentException("periods must be 1..10500");
        Geosim2Lab lab = new Geosim2Lab();
        Geosim2Model model = lab.createModel();
        Simulator simulator = (Simulator) Proxy.newProxyInstance(
            Simulator.class.getClassLoader(), new Class<?>[] {Simulator.class},
            (proxy, method, values) -> {
                if (method.getName().equals("getCurrentTime")) return lab.getCurrentTime();
                if (method.getName().equals("toString")) return "GeoSim reference time bridge";
                throw new UnsupportedOperationException("Archive requested " + method.getName());
            });
        PowerLawSeriesCollector<Geosim2Model> collector = new PowerLawSeriesCollector<>("s",
            new AbstractVariable<Double, Geosim2Model>(Double.class, "War Size") {
                public Double getValue(Geosim2Model current) {
                    int size = War.getWarSize();
                    System.out.println("REFERENCE {\"type\":\"export\",\"time\":"
                        + lab.getCurrentTime().getValue(current).intValue() + ",\"severity\":" + size + "}");
                    return Double.valueOf(size);
                }
            });
        Field collectorField = Geosim2Lab.class.getDeclaredField("powerLawSeriesCollector");
        collectorField.setAccessible(true);
        collectorField.set(lab, collector);
        lab.setup(simulator);
        lab.getCurrentTime().setValue(model, 0.0);
        model.buildModel(simulator);
        snapshot(lab, model);
        for (int period = 1; period <= periods; period++) {
            lab.getCurrentTime().setValue(model, Double.valueOf(period));
            model.step(simulator);
            if (period == 1 || period == 499 || period == 500 || period == 501 || period == periods)
                snapshot(lab, model);
        }
        model.finish(simulator);
        if (lab.getCurrentTime().getValue(model).intValue() != periods)
            throw new IllegalStateException("Reference did not reach requested horizon");
        System.out.println("REFERENCE {\"type\":\"completed\",\"time\":" + periods
            + ",\"active_wars\":" + War.getNum()
            + ",\"queued\":" + War.getNumWars() + "}");
    }
}
