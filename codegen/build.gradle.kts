// Copyright Amazon.com Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

plugins {
    `java-library`
    `maven-publish`
    signing
    //checkstyle
    jacoco
    /*id("com.github.spotbugs") version "4.7.1"*/
    id("io.codearte.nexus-staging") version "0.30.0"
}

allprojects {
    group = "software.amazon.smithy.dafny"
    version = "0.1.0"
}

// This allows smithy-python-codegen to build
// Its build.gradle does not declare its usage of `java-library` nor Maven central,
// and it does not pick it up from its parent project (smithy-python).
subprojects {
    apply(plugin="java-library")

    repositories {
        mavenCentral()
    }
}

// The root project doesn't produce a JAR.
tasks["jar"].enabled = false

// Load the Sonatype user/password for use in publishing tasks.
val sonatypeUser: String? by project
val sonatypePassword: String? by project

/*
 * Sonatype Staging Finalization
 * ====================================================
 *
 * When publishing to Maven Central, we need to close the staging
 * repository and release the artifacts after they have been
 * validated. This configuration is for the root project because
 * it operates at the "group" level.
 */
if (sonatypeUser != null && sonatypePassword != null) {
    apply(plugin = "io.codearte.nexus-staging")

    nexusStaging {
        packageGroup = "software.amazon"
        stagingProfileId = null  // TODO

        username = sonatypeUser
        password = sonatypePassword
    }
}

repositories {
    mavenLocal()
    mavenCentral()
}

subprojects {
    val subproject = this

    /*
     * Java
     * ====================================================
     */
    if (subproject.name != "smithy-dafny-codegen-test"
            && subproject.name != "smithy-python-codegen") {
        if (subproject.name == "smithy-dafny-codegen-cli") {
            apply(plugin = "application")
        } else {
            apply(plugin = "java-library")
        }

        java {
            toolchain {languageVersion.set(JavaLanguageVersion.of(17))}
        }

        tasks.withType<JavaCompile> {
            options.encoding = "UTF-8"
        }

        // Reusable license copySpec
        val licenseSpec = copySpec {
            from("${project.rootDir}/LICENSE")
            from("${project.rootDir}/NOTICE")
        }

        // Set up tasks that build source and javadoc jars.
        tasks.register<Jar>("sourcesJar") {
            metaInf.with(licenseSpec)
            from(sourceSets.main.get().allJava)
            archiveClassifier.set("sources")
        }

        /*tasks.register<Jar>("javadocJar") {
            metaInf.with(licenseSpec)
            from(tasks.javadoc)
            archiveClassifier.set("javadoc")
        }*/

        // Configure jars to include license related info
        tasks.jar {
            metaInf.with(licenseSpec)
            inputs.property("moduleName", subproject.extra["moduleName"])
            manifest {
                attributes["Automatic-Module-Name"] = subproject.extra["moduleName"]
            }
        }

        // Always run javadoc after build.
        /*tasks["build"].finalizedBy(tasks["javadoc"])*/

        /*
         * Maven
         * ====================================================
         */
        apply(plugin = "maven-publish")
        apply(plugin = "signing")

        repositories {
            mavenLocal()
            mavenCentral()
        }

        publishing {
            repositories {
                mavenCentral {
                    url = uri("https://aws.oss.sonatype.org/service/local/staging/deploy/maven2/")
                    credentials {
                        username = sonatypeUser
                        password = sonatypePassword
                    }
                }
            }

            publications {
                create<MavenPublication>("mavenJava") {
                    from(components["java"])

                    // Ship the source and javadoc jars.
                    artifact(tasks["sourcesJar"])
                    /*artifact(tasks["javadocJar"])*/

                    // Include extra information in the POMs.
                    afterEvaluate {
                        pom {
                            name.set(subproject.extra["displayName"].toString())
                            description.set(subproject.description)
                            url.set("https://github.com/awslabs/smithy")
                            licenses {
                                license {
                                    name.set("Apache License 2.0")
                                    url.set("http://www.apache.org/licenses/LICENSE-2.0.txt")
                                    distribution.set("repo")
                                }
                            }
                            developers {
                                developer {
                                    id.set("smithy")
                                    name.set("Smithy")
                                    organization.set("Amazon Web Services")
                                    organizationUrl.set("https://aws.amazon.com")
                                    roles.add("developer")
                                }
                            }
                            scm {
                                url.set("https://github.com/awslabs/smithy.git")
                            }
                        }
                    }
                }
            }
        }

        // Don't sign the artifacts if we didn't get a key and password to use.
        val signingKey: String? by project
        val signingPassword: String? by project
        if (signingKey != null && signingPassword != null) {
            signing {
                useInMemoryPgpKeys(signingKey, signingPassword)
                sign(publishing.publications["mavenJava"])
            }
        }

        /*
         * CheckStyle
         * ====================================================
         */
        //apply(plugin = "checkstyle")

        //tasks["checkstyleTest"].enabled = false

        /*
         * Tests
         * ====================================================
         *
         * Configure the running of tests.
         */
        // Log on passed, skipped, and failed test events if the `-Plog-tests` property is set.
        if (project.hasProperty("log-tests")) {
            tasks.test {
                testLogging.events("passed", "skipped", "failed")
            }
        }

        /*
         * Code coverage
         * ====================================================
         */
        apply(plugin = "jacoco")

        // Always run the jacoco test report after testing.
        tasks["test"].finalizedBy(tasks["jacocoTestReport"])

        // Configure jacoco to generate an HTML report.
        tasks.jacocoTestReport {
            reports {
                xml.isEnabled = false
                csv.isEnabled = false
                html.destination = file("$buildDir/reports/jacoco")
            }
        }

        /*
         * Spotbugs
         * ====================================================
         */
        //apply(plugin = "com.github.spotbugs")

        // We don't need to lint tests.
        //tasks["spotbugsTest"].enabled = false

        // Configure the bug filter for spotbugs.
        /*spotbugs {
            setEffort("max")
            val excludeFile = File("${project.rootDir}/config/spotbugs/filter.xml")
            if (excludeFile.exists()) {
                excludeFilter.set(excludeFile)
            }
        }*/
    }
}
