// Usage: analyzeHeadless <project_dir> <target_name> -postScript RebuildProject.java <workspace> <target> <binary_path>
// Rebuilds a fresh Ghidra project from the original binary (discarding previous state).

import ghidra.app.script.GhidraScript;
import ghidra.app.util.importer.MessageLog;
import ghidra.app.util.importer.ProgramLoader;
import ghidra.app.util.opinion.LoadResults;
import ghidra.framework.model.DomainFolder;
import ghidra.framework.model.Project;
import ghidra.program.model.listing.Program;
import ghidra.app.plugin.core.analysis.AutoAnalysisManager;
import java.io.File;

public class RebuildProject extends GhidraScript {

    @Override
    protected void run() throws Exception {
        if (getScriptArgs().length < 3) {
            throw new IllegalArgumentException("Usage: RebuildProject <workspace> <target> <binary_path>");
        }

        String workspace = getScriptArgs()[0];
        String target = getScriptArgs()[1];
        String binaryPath = getScriptArgs()[2];

        File binary = new File(binaryPath);
        if (!binary.exists()) {
            throw new IllegalArgumentException("Binary not found: " + binaryPath);
        }

        // Close current program if open
        if (currentProgram != null) {
            currentProgram.release(this);
        }

        // Get or create project folder (create intermediate "targets" folder if needed)
        Project project = state.getProject();
        if (project == null) {
            throw new IllegalStateException("No active project");
        }
        DomainFolder rootFolder = project.getProjectData().getRootFolder();
        DomainFolder targetsFolder = rootFolder.getFolder("targets");
        if (targetsFolder == null) {
            targetsFolder = rootFolder.createFolder("targets");
        }
        DomainFolder targetFolder = targetsFolder.getFolder(target);
        if (targetFolder == null) {
            targetFolder = targetsFolder.createFolder(target);
        }

        MessageLog log = new MessageLog();

        // Import binary using ProgramLoader API
        try (LoadResults<Program> loadResults = ProgramLoader.builder()
                .source(binary)
                .project(project)
                .projectFolderPath(targetFolder.getPathname())
                .log(log)
                .monitor(monitor)
                .load()) {
            // getPrimary() returns Loaded<Program> which has save(TaskMonitor) method
            loadResults.getPrimary().save(monitor);

            // Get the program to analyze it
            // Note: We need to get the actual Program object, not just save
            // The Loaded interface provides the program indirectly
        }

        // Re-open the saved program for analysis
        // In headless mode, we need to use the state to get the program
        Program program = state.getCurrentProgram();
        if (program != null) {
            AutoAnalysisManager mgr = AutoAnalysisManager.getAnalysisManager(program);
            mgr.startAnalysis(monitor);
            println("Rebuilt and analyzed: " + binaryPath);
        } else {
            println("Rebuilt project for: " + binaryPath + " (analysis skipped - program not in state)");
        }
    }
}
