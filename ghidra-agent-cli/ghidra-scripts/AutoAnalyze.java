// Usage: analyzeHeadless <project_dir> <target_name> -postScript AutoAnalyze.java <workspace> <target>
// Runs automatic analysis on an imported program.

import ghidra.app.script.GhidraScript;
import ghidra.app.plugin.core.analysis.AutoAnalysisManager;

public class AutoAnalyze extends GhidraScript {

    @Override
    protected void run() throws Exception {
        if (getScriptArgs().length < 2) {
            throw new IllegalArgumentException("Usage: AutoAnalyze <workspace> <target>");
        }

        String target = getScriptArgs()[1];

        // Run auto-analysis using AutoAnalysisManager
        AutoAnalysisManager mgr = AutoAnalysisManager.getAnalysisManager(currentProgram);
        mgr.startAnalysis(monitor);

        println("Auto-analyzed: " + currentProgram.getName());
    }
}
