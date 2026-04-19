// Usage: analyzeHeadless <project_dir> <target_name> -postScript ImportBinary.java <workspace> <target> <binary_path>
// Imports a binary into a Ghidra project.

import ghidra.app.script.GhidraScript;
import ghidra.app.util.importer.MessageLog;
import ghidra.app.util.importer.ProgramLoader;
import ghidra.app.util.opinion.LoadResults;
import ghidra.framework.model.DomainFolder;
import ghidra.framework.model.Project;
import ghidra.program.model.listing.Program;
import java.io.File;

public class ImportBinary extends GhidraScript {

    @Override
    protected void run() throws Exception {
        if (getScriptArgs().length < 3) {
            throw new IllegalArgumentException("Usage: ImportBinary <workspace> <target> <binary_path>");
        }

        String workspace = getScriptArgs()[0];
        String target = getScriptArgs()[1];
        String binaryPath = getScriptArgs()[2];

        File binary = new File(binaryPath);
        if (!binary.exists()) {
            throw new IllegalArgumentException("Binary not found: " + binaryPath);
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

        // Import using ProgramLoader API - same pattern as Ghidra's ImportAllProgramsFromADirectoryScript
        try (LoadResults<Program> loadResults = ProgramLoader.builder()
                .source(binary)
                .project(project)
                .projectFolderPath(targetFolder.getPathname())
                .log(log)
                .monitor(monitor)
                .load()) {
            // getPrimary() returns Loaded<Program> which has save(TaskMonitor) method
            loadResults.getPrimary().save(monitor);
            println("Imported: " + binaryPath);
        }

        println(log.toString());
    }
}
